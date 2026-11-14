#![feature(box_patterns)]

#[macro_use]
extern crate lazy_static;

use base64::{engine::general_purpose, Engine as _};
use std::fmt;
use std::path::PathBuf;
use swc_common::comments::SingleThreadedComments;
use swc_common::source_map::SourceMapGenConfig;
use swc_common::{self, sync::Lrc, FileName, Mark, SourceFile, SourceMap};
use swc_core::common::GLOBALS;
use swc_ecma_ast::{
    Ident, ImportDecl, ImportNamedSpecifier, ImportSpecifier, Module, ModuleDecl, ModuleExportName,
    ModuleItem,
};
use swc_ecma_codegen::Emitter;
use swc_ecma_parser::TsSyntax;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecma_transforms::resolver;
use swc_ecma_utils::private_ident;
use swc_ecma_visit::{VisitMutWith, VisitWith};

mod bindings;
mod locate;
mod snippets;
mod transform;

#[derive(Default)]
pub struct Options {
    pub filename: Option<PathBuf>,
    pub inline_source_map: bool,
}

/// Reads `<template>` content tags. It holds no state, so one preprocessor can
/// be reused for any number of files. Each call to `parse` or `process` keeps
/// its source map and comments to itself, and drops them when it returns.
#[derive(Default)]
pub struct Preprocessor {}

/// A parse error, with the source map needed to show where it is.
pub struct ParseError {
    pub error: swc_ecma_parser::error::Error,
    pub source_map: Lrc<SourceMap>,
}

// `SourceMap` does not implement `Debug`, so only the error is shown.
impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)
    }
}

impl From<ParseError> for swc_ecma_parser::error::Error {
    fn from(err: ParseError) -> Self {
        err.error
    }
}

pub struct CodeMapPair {
    pub code: String,
    pub map: String,
}

pub const IMPORT_ALIAS: &str = "template_fd9b2463e5f141cfb5666b64daa1f11a";

struct SourceMapConfig;
impl SourceMapGenConfig for SourceMapConfig {
    fn file_name_to_source(&self, f: &swc_common::FileName) -> String {
        f.to_string()
    }

    fn inline_sources_content(&self, _: &swc_common::FileName) -> bool {
        true
    }
}

impl Preprocessor {
    pub fn new() -> Self {
        Self {}
    }

    fn lexer<'a>(
        &self,
        source_file: &'a SourceFile,
        comments: &'a SingleThreadedComments,
    ) -> Lexer<'a> {
        Lexer::new(
            Syntax::Typescript(TsSyntax {
                decorators: true,
                ..Default::default()
            }),
            Default::default(),
            StringInput::from(source_file),
            Some(comments),
        )
    }

    pub fn parse(
        &self,
        src: &str,
        options: Options,
    ) -> Result<Vec<locate::Occurrence>, ParseError> {
        let source_map: Lrc<SourceMap> = Default::default();
        let comments = SingleThreadedComments::default();
        let source_file = new_source_file(&source_map, src, options.filename);

        let mut parser = Parser::new_from(self.lexer(&source_file, &comments));
        GLOBALS.set(&Default::default(), || {
            let parsed_module = parser.parse_module().map_err(|error| ParseError {
                error,
                source_map: source_map.clone(),
            })?;

            let mut visitor =
                locate::LocateContentTagVisitor::new(src.to_string(), source_file.start_pos);

            parsed_module.visit_with(&mut visitor);

            Ok(visitor.occurrences)
        })
    }

    pub fn process(&self, src: &str, options: Options) -> Result<CodeMapPair, ParseError> {
        let target_specifier = "template";
        let target_module = "@ember/template-compiler";
        let source_map: Lrc<SourceMap> = Default::default();
        let comments = SingleThreadedComments::default();
        let source_file = new_source_file(&source_map, src, options.filename);

        let mut parser = Parser::new_from(self.lexer(&source_file, &comments));
        GLOBALS.set(&Default::default(), || {
            let mut parsed_module = parser.parse_module().map_err(|error| ParseError {
                error,
                source_map: source_map.clone(),
            })?;

            let id = private_ident!(IMPORT_ALIAS);
            let mut needs_import = false;
            parsed_module.visit_mut_with(&mut transform::TransformVisitor::new(
                &id,
                Some(&mut needs_import),
            ));

            if needs_import {
                insert_import(&mut parsed_module, target_module, target_specifier, &id)
            }

            let unresolved_mark = Mark::new();
            let top_level_mark = Mark::new();

            parsed_module.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, false));

            Ok(print(
                &parsed_module,
                &source_map,
                &comments,
                options.inline_source_map,
            ))
        })
    }
}

fn new_source_file(
    source_map: &Lrc<SourceMap>,
    src: &str,
    filename: Option<PathBuf>,
) -> Lrc<SourceFile> {
    let filename = match filename {
        Some(name) => FileName::Real(name),
        None => FileName::Anon,
    };
    source_map.new_source_file(filename.into(), src.to_string())
}

fn print(
    module: &Module,
    source_map: &Lrc<SourceMap>,
    comments: &SingleThreadedComments,
    inline_source_map: bool,
) -> CodeMapPair {
    let mut buf = vec![];
    let mut srcmap = vec![];
    let mut source_map_buffer = vec![];
    let mut emitter = Emitter {
        cfg: Default::default(),
        cm: source_map.clone(),
        wr: swc_ecma_codegen::text_writer::JsWriter::new(
            source_map.clone(),
            "\n",
            &mut buf,
            Some(&mut srcmap),
        ),
        comments: Some(comments),
    };
    emitter.emit_module(module).unwrap();

    source_map
        .build_source_map(&srcmap, None, SourceMapConfig {})
        .to_writer(&mut source_map_buffer)
        .unwrap();

    if inline_source_map {
        let mut comment = "//# sourceMappingURL=data:application/json;base64,"
            .to_owned()
            .into_bytes();
        buf.append(&mut comment);

        let mut encoded = general_purpose::URL_SAFE_NO_PAD
            .encode(source_map_buffer.clone())
            .into_bytes();

        buf.append(&mut encoded);
    }

    let s = String::from_utf8_lossy(&buf);

    CodeMapPair {
        code: s.to_string(),
        map: String::from_utf8(source_map_buffer.clone()).unwrap(),
    }
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
                imported: Some(ModuleExportName::Ident(Ident::new_no_ctxt(
                    target_specifier.into(),
                    Default::default(),
                ))),
                is_type_only: false,
            })],
            src: Box::new(target_module.into()),
            type_only: false,
            with: None,
            phase: Default::default(),
        })),
    );
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
  r#"let x = <template>hello</template>"#,
  r#"import { template as template_UUID } from "@ember/template-compiler";
     let x = template_UUID(`hello`, { eval() { return eval(arguments[0])} });"#
}

testcase! {
  preexisting_import,
  r#"import { template } from "@ember/template-compiler";
     let x = <template>hello</template>"#,
  r#"import { template as template_UUID } from "@ember/template-compiler";
     import { template } from "@ember/template-compiler";
     let x = template_UUID(`hello`, { eval() { return eval(arguments[0])} });"#
}

testcase! {
  preexisting_renamed_import,
  r#"import { template as t } from "@ember/template-compiler";
     let x = <template>hello</template>"#,
  r#"import { template as template_UUID } from "@ember/template-compiler";
     import { template as t } from "@ember/template-compiler";
     let x = template_UUID(`hello`, { eval() { return eval(arguments[0])} })"#
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
  r#"import { template as template_UUID } from "@ember/template-compiler";
     function template() {};
     console.log(template());
     export default template_UUID(`Hi`, { eval() { return eval(arguments[0])} });"#
}

testcase! {
  avoids_local_collision,
  r#"export default function (template) {
         console.log(template);
         return <template>X</template>;
       };"#,
  r#"import { template as template_UUID } from "@ember/template-compiler";
       export default function(template) {
         console.log(template);
         return template_UUID(`X`, { eval() { return eval(arguments[0])} });
       };"#
}

testcase! {
  handles_typescript,
  r#"function makeComponent(message: string) {
        console.log(message);
        return <template>hello</template>
    }"#,
  r#"import { template as template_UUID } from "@ember/template-compiler";
       function makeComponent(message: string) {
         console.log(message);
         return template_UUID(`hello`, { eval() { return eval(arguments[0]) } });
       }"#
}

testcase! {
  handles_typescript_implied_export_default,
  r#"<template>hello</template> satisfies MyType"#,
  r#"import { template as template_UUID } from "@ember/template-compiler";
export default template_UUID(`hello`, {
    eval () {
        return eval(arguments[0]);
    }
}) satisfies MyType;"#
}

testcase! {
  handles_typescript_this,
  r#"function f(this: Context, ...args: unknown[]) {
        function t(this: Context, ...args: unknown[]) {};
        return <template></template>
    }"#,
  r#"import { template as template_UUID } from "@ember/template-compiler";
       function f(this: Context, ...args: unknown[]) {
         function t(this: Context, ...args: unknown[]) {}
         ;
         return template_UUID(``, { eval() { return eval(arguments[0]) } });
       }"#
}

#[test]
fn preserves_type_reexport() -> Result<(), swc_ecma_parser::error::Error> {
    // we're using a substring test here because the bug I'm testing also effects
    // the normalizer that the other tests are relying on.
    test_helpers::testcase_substr("export { type X } from 'elsewhere';", "type X")
}

testcase! {
  extraneous_indentation_strip,
  r#"let x = <template>
    hello
  </template>"#,
  r#"import { template as template_UUID } from "@ember/template-compiler";
     let x = template_UUID(`hello`, { eval() { return eval(arguments[0])} });"#
}

testcase! {
  multiline_extraneous_indentation_strip,
  r#"let x = <template>
    hello

    extra line break
    <div>
      content
    </div>
  </template>"#,
  r#"import { template as template_UUID } from "@ember/template-compiler";
     let x = template_UUID(`hello

extra line break
<div>
  content
</div>`, { eval() { return eval(arguments[0])} });"#
}

#[test]
fn reused_preprocessor_gives_the_same_output() {
    let p = Preprocessor::new();
    let src = "let ü = 1;\nlet x = <template>💩</template>";
    let first = p.process(src, Default::default()).unwrap();
    for _ in 0..3 {
        p.process("let y = <template>other</template>", Default::default())
            .unwrap();
        let again = p.process(src, Default::default()).unwrap();
        assert_eq!(again.code, first.code);
        assert_eq!(again.map, first.map);
    }
}

#[test]
fn reused_preprocessor_reports_errors_in_the_right_file() {
    let p = Preprocessor::new();
    p.process("let x = 1;\nlet y = 2;\n", Default::default())
        .unwrap();
    let err = p.process("let x = ;", Default::default()).err().unwrap();
    assert_eq!(
        err.source_map
            .span_to_string(swc_common::Spanned::span(&err.error)),
        "<anon>:1:9: 1:10"
    );
}
