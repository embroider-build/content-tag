#![feature(box_patterns)]

#[macro_use]
extern crate lazy_static;

use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;
use std::path::PathBuf;
use swc_common::comments::SingleThreadedComments;
use swc_common::source_map::SourceMapGenConfig;
use swc_common::{self, sync::Lrc, FileName, SourceMap};
use swc_common::{Mark, Span};
use swc_core::common::GLOBALS;
use swc_ecma_ast::{
    Callee, ClassMember, ContentTagContent, ContentTagEnd, ContentTagExpression, ContentTagMember,
    ContentTagStart, Decl, Ident, ImportDecl, ImportNamedSpecifier, ImportSpecifier, Module,
    ModuleDecl, ModuleExportName, ModuleItem, VarDecl, VarDeclarator,
};
use swc_ecma_codegen::Emitter;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};
use swc_ecma_transforms::hygiene::hygiene_with_config;
use swc_ecma_transforms::resolver;
use swc_ecma_utils::private_ident;
use swc_ecma_visit::{as_folder, Visit, VisitMutWith, VisitWith};

mod bindings;
mod snippets;
mod transform;

#[derive(Default)]
pub struct Options {
    pub filename: Option<PathBuf>,
    pub inline_source_map: bool,
}

pub struct Preprocessor {
    source_map: Lrc<SourceMap>,
    comments: SingleThreadedComments,
}

struct SourceMapConfig;
impl SourceMapGenConfig for SourceMapConfig {
    fn file_name_to_source(&self, f: &swc_common::FileName) -> String {
        f.to_string()
    }

    fn inline_sources_content(&self, _: &swc_common::FileName) -> bool {
        true
    }
}

#[derive(Default, Debug)]
struct ContentTagVisitor {
    occurrences: Vec<Occurrence>,
}

impl ContentTagVisitor {
    fn add_occurrence(
        &mut self,
        span: &Span,
        opening: &ContentTagStart,
        contents: &ContentTagContent,
        closing: &ContentTagEnd,
    ) {
        let occurrence = Occurrence {
            range: span.into(),
            content_range: contents.span.into(),
            end_range: closing.span.into(),
            start_range: opening.span.into(),
            tag_name: "template".to_owned(),
            kind: "template-tag".to_owned(),
            contents: contents.value.to_string(),
        };

        self.occurrences.push(occurrence);
    }
}

impl Visit for ContentTagVisitor {
    fn visit_expr(&mut self, n: &swc_ecma_ast::Expr) {
        match n {
            swc_ecma_ast::Expr::ContentTagExpression(ContentTagExpression {
                span,
                opening,
                contents,
                closing,
            }) => {
                self.add_occurrence(span, opening, contents, closing);
            }
            _ => {}
        }
    }

    fn visit_class_member(&mut self, n: &ClassMember) {
        match n {
            ClassMember::ContentTagMember(ContentTagMember {
                span,
                opening,
                contents,
                closing,
            }) => {
                self.add_occurrence(span, opening, contents, closing);
            }
            _ => {}
        }
    }

    // FIXME: is this already covered by visit_class_member?
    fn visit_class_members(&mut self, n: &[ClassMember]) {
        swc_ecma_visit::visit_class_members(self, n)
    }
}

impl Preprocessor {
    pub fn new() -> Self {
        Self {
            source_map: Default::default(),
            comments: SingleThreadedComments::default(),
        }
    }

    pub fn parse(&self, src: &str) -> Result<Vec<Occurrence>, swc_ecma_parser::error::Error> {
        let target_specifier = "template";
        let options: Options = Default::default();
        let target_module = "@ember/template-compiler";
        let filename = match options.filename {
            Some(name) => FileName::Real(name),
            None => FileName::Anon,
        };

        let source_file = self.source_map.new_source_file(filename, src.to_string());

        let lexer = Lexer::new(
            Syntax::Typescript(TsConfig {
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

            let mut visitor = ContentTagVisitor::default();

            parsed_module.visit_with(&mut visitor);

            Ok(visitor.occurrences)
        })
    }

    pub fn process(
        &self,
        src: &str,
        options: Options,
    ) -> Result<String, swc_ecma_parser::error::Error> {
        let target_specifier = "template";
        let target_module = "@ember/template-compiler";
        let filename = match options.filename {
            Some(name) => FileName::Real(name),
            None => FileName::Anon,
        };

        let source_file = self.source_map.new_source_file(filename, src.to_string());

        let lexer = Lexer::new(
            Syntax::Typescript(TsConfig {
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
                ignore_eval: false,
            });
            parsed_module.visit_mut_with(&mut h);

            simplify_imports(&mut parsed_module);

            Ok(self.print(&parsed_module, options.inline_source_map))
        })
    }

    fn print(&self, module: &Module, inline_source_map: bool) -> String {
        let mut buf = vec![];
        let mut srcmap = vec![];
        let mut emitter = Emitter {
            cfg: Default::default(),
            cm: self.source_map.clone(),
            wr: swc_ecma_codegen::text_writer::JsWriter::new(
                self.source_map().clone(),
                "\n",
                &mut buf,
                Some(&mut srcmap),
            ),
            comments: Some(&self.comments),
        };
        emitter.emit_module(module).unwrap();

        if inline_source_map {
            let mut source_map_buffer = vec![];
            self.source_map()
                .build_source_map_with_config(&srcmap, None, SourceMapConfig {})
                .to_writer(&mut source_map_buffer)
                .unwrap();

            let mut comment = "//# sourceMappingURL=data:application/json;base64,"
                .to_owned()
                .into_bytes();
            buf.append(&mut comment);

            let mut encoded = general_purpose::URL_SAFE_NO_PAD
                .encode(source_map_buffer)
                .into_bytes();

            buf.append(&mut encoded);
        }

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

#[derive(Serialize, Debug, Eq, PartialEq)]
pub struct Range {
    start: usize,
    end: usize,
}

impl From<&Span> for Range {
    fn from(value: &Span) -> Self {
        Range {
            start: value.lo.0 as usize - 1,
            end: value.hi.0 as usize - 1,
        }
    }
}

impl From<Span> for Range {
    fn from(value: Span) -> Self {
        (&value).into()
    }
}

#[derive(Serialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Occurrence {
    range: Range,
    content_range: Range,
    contents: String,
    // the span of the closing "</template>" tag
    end_range: Range,
    // the span of the opening "<template>" tag
    start_range: Range,
    tag_name: String,
    #[serde(rename = "type")]
    kind: String, // FIXME enum?
}

#[cfg(test)]
mod test_helpers;

#[cfg(test)]
mod parser_tests {
    use crate::{Occurrence, Preprocessor, Range};

    #[test]
    fn test_basic_example() {
        let p = Preprocessor::new();
        let output = p.parse("<template>Hello!</template>").unwrap();
        let expected = Occurrence {
            kind: "template-tag".into(),
            tag_name: "template".into(),
            contents: "Hello!".into(),
            range: Range { start: 0, end: 27 },
            content_range: Range { start: 10, end: 16 },
            start_range: Range { start: 0, end: 10 },
            end_range: Range { start: 16, end: 27 },
        };
        assert_eq!(output, vec![expected]);
    }

    #[test]
    fn test_expression_position() {
        let p = Preprocessor::new();
        let output = p.parse("const tpl = <template>Hello!</template>").unwrap();

        let expected = vec![Occurrence {
            kind: "template-tag".into(),
            tag_name: "template".into(),
            contents: "Hello!".into(),
            range: Range { start: 12, end: 39 },
            content_range: Range { start: 22, end: 28 },
            start_range: Range { start: 12, end: 22 },
            end_range: Range { start: 28, end: 39 },
        }];

        assert_eq!(output, expected);
    }

    #[test]
    fn test_inside_class_body() {
        let p = Preprocessor::new();
        let src = r#"
                    class A {
                      <template>Hello!</template>
                    }
                "#;

        let bytes = src.as_bytes();
        let output = p.parse(src).unwrap();

        let expected = vec![Occurrence {
            kind: "template-tag".into(),
            tag_name: "template".into(),
            contents: "Hello!".into(),
            range: Range { start: 53, end: 80 },
            content_range: Range { start: 63, end: 69 },
            start_range: Range { start: 53, end: 63 },
            end_range: Range { start: 69, end: 80 },
        }];

        assert_eq!(output, expected);
    }

    #[test]
    fn test_preceded_by_a_slash_character() {
        let p = Preprocessor::new();
        // What is this testing?
        // Would a better test be:
        // `const divide = 1 / <template>Hello!</template>;`
        let output = p
            .parse(
                r#"
                  const divide = () => 4 / 2;
                  <template>Hello!</template>
                "#,
            )
            .unwrap();

        let expected = vec![Occurrence {
            range: Range { start: 65, end: 92 },
            content_range: Range { start: 75, end: 81 },
            contents: "Hello!".into(),
            end_range: Range { start: 81, end: 92 },
            start_range: Range { start: 65, end: 75 },
            tag_name: "template".into(),
            kind: "template-tag".into(),
        }];

        assert_eq!(output, expected);
    }

    #[test]
    fn test_template_inside_a_regexp() {
        let p = Preprocessor::new();
        let output = p
            .parse(
                r#"
                  const myregex = /<template>/;
                  <template>Hello!</template>
                "#,
            )
            .unwrap();

        let expected = vec![Occurrence {
            range: Range { start: 67, end: 94 },
            content_range: Range { start: 77, end: 83 },
            contents: "Hello!".into(),
            end_range: Range { start: 83, end: 94 },
            start_range: Range { start: 67, end: 77 },
            tag_name: "template".into(),
            kind: "template-tag".into(),
        }];

        assert_eq!(output, expected);
    }

    #[test]
    fn test_no_match() {
        let p = Preprocessor::new();
        let output = p.parse("console.log('Hello world');").unwrap();

        assert_eq!(output, vec![]);
    }
}

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
  r#"let x = <template>hello</template>"#,
  r#"import { template } from "@ember/template-compiler";
     let x = template(`hello`, { eval() { return eval(arguments[0])} });"#
}

testcase! {
  uses_preexisting_import,
  r#"import { template } from "@ember/template-compiler";
     let x = <template>hello</template>"#,
  r#"import { template } from "@ember/template-compiler";
     let x = template(`hello`, { eval() { return eval(arguments[0])} });"#
}

testcase! {
  uses_preexisting_renamed_import,
  r#"import { template as t } from "@ember/template-compiler";
     let x = <template>hello</template>"#,
  r#"import { template as t } from "@ember/template-compiler";
     let x = t(`hello`, { eval() { return eval(arguments[0])} })"#
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
     export default template1(`Hi`, { eval() { return eval(arguments[0])} });"#
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
         return template1(`X`, { eval() { return eval(arguments[0])} });
       };"#
}

testcase! {
  handles_typescript,
  r#"function makeComponent(message: string) {
        console.log(message);
        return <template>hello</template>
    }"#,
  r#"import { template } from "@ember/template-compiler";
       function makeComponent(message: string) {
         console.log(message);
         return template(`hello`, { eval() { return eval(arguments[0]) } });
       }"#
}
