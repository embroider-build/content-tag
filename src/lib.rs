use std::path::PathBuf;
use swc_common::comments::SingleThreadedComments;
use swc_common::Mark;
use swc_common::{self, sync::Lrc, FileName, SourceMap};
use swc_core::common::GLOBALS;
use swc_ecma_ast::{
    Expr, Ident, ImportDecl, ImportNamedSpecifier, ImportSpecifier, Module, ModuleDecl,
    ModuleExportName, ModuleItem,
};
use swc_ecma_codegen::Emitter;
use swc_ecma_parser::{lexer::Lexer, EsConfig, Parser, StringInput, Syntax};
use swc_ecma_transforms::hygiene::hygiene_with_config;
use swc_ecma_transforms::resolver;
use swc_ecma_utils::private_ident;
use swc_ecma_visit::{as_folder, VisitMutWith};

mod bindings;
mod transform;

#[derive(Default)]
pub struct Options {
    pub filename: Option<PathBuf>,
}

pub struct Preprocessor {
    source_map: Lrc<SourceMap>,
    comments: SingleThreadedComments,
}

impl Preprocessor {
    pub fn new() -> Self {
        Self {
            source_map: Default::default(),
            comments: SingleThreadedComments::default(),
        }
    }

    pub fn process(
        &self,
        src: &str,
        options: Options,
    ) -> Result<String, swc_ecma_parser::error::Error> {
        let target_specifier = "template";
        let target_module = "@ember/template-compiler";
        let filename = options.filename.unwrap_or_else(|| "anonymous".into());

        let source_file = self
            .source_map
            .new_source_file(FileName::Real(filename), src.to_string());

        let lexer = Lexer::new(
            Syntax::Es(EsConfig {
                decorators: true,
                ..Default::default()
            }),
            Default::default(),
            StringInput::from(&*source_file),
            Some(&self.comments),
        );
        let mut parser = Parser::new_from(lexer);
        GLOBALS.set(&Default::default(), || {
            let mut parsed_module = parser.parse_module()?;

            let found_id = find_existing_import(&parsed_module, target_module, target_specifier);
            let had_id_already = found_id.is_some();
            let id = found_id.unwrap_or_else(|| private_ident!(target_specifier));
            let mut needs_import = false;
            parsed_module.visit_mut_with(&mut as_folder(transform::TransformVisitor::new(
                &id,
                Some(&mut needs_import),
                self.parse_expression(r#"({ eval() { return eval(arguments[0]); } })"#),
            )));

            if !had_id_already && needs_import {
                insert_import(&mut parsed_module, target_module, target_specifier, &id)
            }

            let unresolved_mark = Mark::new();
            let top_level_mark = Mark::new();

            parsed_module.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, false));

            let mut h = hygiene_with_config(swc_ecma_transforms::hygiene::Config {
                keep_class_names: true,
                top_level_mark,
                safari_10: false,
            });
            parsed_module.visit_mut_with(&mut h);

            simplify_imports(&mut parsed_module);

            Ok(self.print(&parsed_module))
        })
    }

    fn print(&self, module: &Module) -> String {
        let mut buf = vec![];
        let mut emitter = Emitter {
            cfg: Default::default(),
            cm: self.source_map.clone(),
            wr: Box::new(swc_ecma_codegen::text_writer::JsWriter::new(
                self.source_map.clone(),
                "\n",
                &mut buf,
                None,
            )),
            comments: Some(&self.comments),
        };
        emitter.emit_module(module).unwrap();
        let s = String::from_utf8_lossy(&buf);
        s.to_string()
    }

    pub fn source_map(&self) -> Lrc<SourceMap> {
        return self.source_map.clone();
    }

    fn parse_expression(&self, src: &str) -> Box<Expr> {
        let filename = "glimmer-template-prelude.js".into();
        let source_file = self
            .source_map
            .new_source_file(FileName::Real(filename), src.to_string());

        let lexer = Lexer::new(
            Syntax::Es(EsConfig {
                decorators: true,
                ..Default::default()
            }),
            Default::default(),
            StringInput::from(&*source_file),
            Some(&self.comments),
        );

        let mut parser = Parser::new_from(lexer);
        let module = parser.parse_module().unwrap();

        module
            .body
            .first()
            .unwrap()
            .as_stmt()
            .unwrap()
            .as_expr()
            .unwrap()
            .expr
            .as_paren()
            .unwrap()
            .expr
            .clone()
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
            asserts: None,
        })),
    );
}

// It's not until after the hygiene pass that we know what local name is being
// used for our import. If it turns out to equal the imported name, we can
// implify from "import { template as template } from..." down to  "import {
// template } from ...".
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

#[cfg(test)]
mod test_helpers;

macro_rules! testcase {
    ($test_name:ident, $input:expr, $expected:expr) => {
        #[test]
        fn $test_name() -> Result<(), swc_ecma_parser::error::Error> {
            test_helpers::testcase($input, $expected)
        }
    };
}

testcase! {
  no_preexisting_import,
  r#"<template>hello</template>"#,
  r#"import { template } from "@ember/template-compiler";
     template("hello", { eval() { return eval(arguments[0])} });"#
}

testcase! {
  uses_preexisting_import,
  r#"import { template } from "@ember/template-compiler";
     <template>hello</template>"#,
  r#"import { template } from "@ember/template-compiler";
     template("hello", { eval() { return eval(arguments[0])} });"#
}

testcase! {
  uses_preexisting_renamed_import,
  r#"import { template as t } from "@ember/template-compiler";
     <template>hello</template>"#,
  r#"import { template as t } from "@ember/template-compiler";
     t("hello", { eval() { return eval(arguments[0])} })"#
}

testcase! {
  no_template_tags,
  r#"console.log('hello')"#,
  r#"console.log('hello')"#
}

testcase! {
  avoids_top_level_collision,
  r#"function template() {};
     console.log(template());
     export default <template>Hi</template>"#,
  r#"import { template as template1 } from "@ember/template-compiler";
     function template() {};
     console.log(template());
     export default template1("Hi", { eval() { return eval(arguments[0])} });"#
}

testcase! {
  avoids_local_collision,
  r#"export default function (template) {
         console.log(template);
         return <template>X</template>; 
       };"#,
  r#"import { template as template1 } from "@ember/template-compiler";
       export default function(template) {
         console.log(template);
         return template1("X", { eval() { return eval(arguments[0])} });
       };"#
}
