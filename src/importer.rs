use swc_common::Mark;
use swc_ecma_ast::{
    Ident, ImportDecl, ImportNamedSpecifier, ImportSpecifier, Module, ModuleDecl, ModuleExportName,
    ModuleItem,
};
use swc_ecma_transforms_base::{
    hygiene::{hygiene_with_config, Config},
    resolver,
};
use swc_ecma_utils::private_ident;
use swc_ecma_visit::{Fold, VisitMut, VisitMutWith};

/// Logic for managing the insertion of the following import statement:
///
/// ```js
/// import { template } from "@ember/template";
/// ```
///
/// * We only want to insert this if there are any `<template>` tags in the
///   source file.
/// * If this is already imported in the source file for some reason, we want
///   to reuse it instead of duplicating it.
/// * We may have to pick a unique name (`import { template as X }`), such that
///   X can be successfully referenced from all the locations where there is a
///   `<template>` tag. For example, the naive `template` name won't work here:
///
///   ```gjs
///   function foo(template, ...args) {
///     return <template>...</template>;
///   }
///   ```
///
///   This is a fairly trivial case, but the naming collision ("shadowing") can
///   occur anywhere along the scope chain. Alternatively, the top-level scope
///   may also already have another unrelated import/variable named `template`.
pub struct Importer {
    top_level_mark: Mark,
    target_module: &'static str,
    target_specifier: &'static str,
    id: Ident,
    need_insert: bool,
}

impl Importer {
    pub fn prepare(
        parsed_module: &mut Module,
        target_module: &'static str,
        target_specifier: &'static str,
    ) -> Self {
        // First, we need to prepare the AST by attaching "SyntaxContext" to
        // identifiers, basically adding scope information to help uniquely
        // distinguish variables that otherwise have the same names, i.e.
        // differentiating "shadowed" variables.
        //
        // According to the  documentation, the resolver expects a "clean" AST
        // where none of the identifiers already have a pre-existing syntax
        // context attached to them, so we need to run this immediately after
        // parsing before we insert our own private identifier for the import.
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();
        parsed_module.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, false));

        // Look for an existing import statement for the target module and
        // specifier, possibly aliased into a different local name. If found,
        // we can reuse its identifier, otherwise, make a new "private"
        // identifier for it, which is an identifier with the desired name but
        // attached to a unique "SyntaxContext".
        let (id, need_insert) =
            match find_existing_import(&parsed_module, target_module, target_specifier) {
                Some(id) => (id, false),
                None => (private_ident!(target_specifier), true),
            };

        Self {
            top_level_mark,
            target_module,
            target_specifier,
            id,
            need_insert,
        }
    }

    pub fn id(&self) -> &Ident {
        &self.id
    }

    //     let mut r = renamer(swc_ecma_transforms::hygiene::Config {
    //         keep_class_names: true,
    //         top_level_mark,
    //         safari_10: false,
    //         ignore_eval: false,
    //     });
    //     parsed_module.visit_mut_with(&mut h);

    //     simplify_imports(&mut parsed_module);
    // }
    pub fn insert(self, parsed_module: &mut Module) {
        // First, insert the import statement, if needed:
        //
        // ```js
        // import { $target_specifier as $ID } from $target_module;
        // ```
        if self.need_insert {
            insert_import(
                parsed_module,
                self.target_module,
                self.target_specifier,
                &self.id,
            );
        }

        // Earlier, we may have made a unique SWC identifier for the import, or
        // we may have reused the same identifier from an existing import.
        //
        // Either way, because we ran the "resolver" step, all the identifiers
        // have a "SyntaxContext" attached to it and are unique. SWC uniquely
        // identify each identifier by both their JavaScript name AND also the
        // "SyntaxContext" they came from.
        //
        // This allows our transformer to use that identifier in arbitrarily
        // deeply nested code and SWC will still refer to the top-level import
        // regardless of whether it is "shadowed" by a local scope higher up.
        //
        // You can think of this as internally representing each variable like
        // so:
        //
        // ```js
        // let foo;
        //
        // function bar(foo) {
        //   console.log(foo);
        // }
        //
        // console.log(foo);
        // ```
        //
        // Becomes:
        //
        // ```js
        // let foo__top_level;
        //
        // function bar(foo__function_bar) {
        //   console.log(foo__function_bar);
        // }
        //
        // console.log(foo__top_level);
        // ```
        //
        // However, this system only work within SWC. "SyntaxContext" is just
        // an internal extension in the AST and not actually reflected in the
        // JavaScript names in any way. We are about to write things back out
        // as normal JavaScript code, so this won't help us.
        //
        // In SWC, you are expect to run a "hygiene" rename pass that go find
        // these kind of name collisions and actually rename the JS identifier
        // names.
        //
        // Specifically, the first^ occurrence of a variable name gets to keep
        // its name intact, any any subsequent variables with the same name
        // (potentially "shadowing" an outer variable) gets renamed as
        // `${name}${n++}`, like so:
        //
        // ```js
        // let foo;
        //
        // function bar(foo1) {
        //   console.log(foo1);
        // }
        //
        // console.log(foo);
        // ```
        //
        // ^ "first" doesn't imply any particular traversal order, so don't
        // go assuming that the top-level name will be the bare identifier,
        // in fact, it often isn't.
        let mut renamer = import_renamer(self.top_level_mark);
        parsed_module.visit_mut_with(&mut renamer);

        // Finally, we know what local name is being used for our import. If it
        // turns out that we didn't have to rename it (which is probably most
        // of the time), then we can rename the `import { foo as foo }` to just
        // `import { foo }`.
        simplify_imports(parsed_module);
    }
}

fn find_existing_import(
    parsed_module: &Module,
    target_module: &str,
    target_specifier: &str,
) -> Option<Ident> {
    for item in parsed_module.body.iter() {
        match item {
            ModuleItem::ModuleDecl(ModuleDecl::Import(import_declaration)) => {
                if import_declaration.src.value.to_string() == target_module {
                    for specifier in import_declaration.specifiers.iter() {
                        match specifier {
                            ImportSpecifier::Named(s) => {
                                let imported = match &s.imported {
                                    Some(ModuleExportName::Ident(i)) => i.sym.to_string(),
                                    Some(ModuleExportName::Str(s)) => s.value.to_string(),
                                    None => s.local.sym.to_string(),
                                };
                                if imported == target_specifier {
                                    return Some(s.local.clone());
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn insert_import(
    parsed_module: &mut Module,
    target_module: &str,
    target_specifier: &str,
    local: &Ident,
) {
    parsed_module.body.insert(
        0,
        ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
            span: Default::default(),
            specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
                span: Default::default(),
                local: local.clone(),
                imported: Some(ModuleExportName::Ident(Ident::new(
                    target_specifier.into(),
                    Default::default(),
                ))),
                is_type_only: false,
            })],
            src: Box::new(target_module.into()),
            type_only: false,
            with: None,
        })),
    );
}

fn simplify_imports(parsed_module: &mut Module) {
    for item in parsed_module.body.iter_mut() {
        match item {
            ModuleItem::ModuleDecl(ModuleDecl::Import(import_declaration)) => {
                for specifier in import_declaration.specifiers.iter_mut() {
                    match specifier {
                        ImportSpecifier::Named(specifier) => {
                            if let ImportNamedSpecifier {
                                imported: Some(ModuleExportName::Ident(imported)),
                                local,
                                ..
                            } = specifier
                            {
                                if local.sym == imported.sym {
                                    specifier.imported = None;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

// Based on swc_ecma_transforms_base::hygiene

fn import_renamer(top_level_mark: Mark) -> impl 'static + Fold + VisitMut {
    hygiene_with_config(Config {
        keep_class_names: true,
        top_level_mark,
        safari_10: false,
        ignore_eval: false,
    })
}
