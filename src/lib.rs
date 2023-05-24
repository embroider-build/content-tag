use std::path::PathBuf;
use swc_common::comments::SingleThreadedComments;
use swc_common::{self, sync::Lrc, FileName, SourceMap};
use swc_core::common::GLOBALS;
use swc_core::ecma::transforms::base::hygiene::hygiene;
use swc_ecma_ast::{
    Ident, ImportDecl, ImportNamedSpecifier, ImportSpecifier, Module, ModuleDecl, ModuleExportName,
    ModuleItem,
};
use swc_ecma_codegen::Emitter;
use swc_ecma_parser::{lexer::Lexer, EsConfig, Parser, StringInput, Syntax};
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

            let id = find_existing_import(&parsed_module, "@ember/template-compiler", "template")
                .unwrap_or_else(|| {
                    insert_import(&mut parsed_module, "@ember/template-compiler", "template")
                });

            let mut tr = as_folder(transform::TransformVisitor::new(&id));
            parsed_module.visit_mut_with(&mut tr);

            let mut h = hygiene();
            parsed_module.visit_mut_with(&mut h);

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

fn insert_import(parsed_module: &mut Module, target_module: &str, target_specifier: &str) -> Ident {
    let local = private_ident!("template");
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
    local
}
#[test]
fn existing_import() -> Result<(), swc_ecma_parser::error::Error> {
    let p = Preprocessor::new();
    assert_eq!(p.process("<template>hello</template>", Default::default())?, "");
    Ok(())
}
