#![feature(box_patterns)]

#[macro_use]
extern crate lazy_static;

use base64::{engine::general_purpose, Engine as _};
use std::path::PathBuf;
use swc_common::comments::SingleThreadedComments;
use swc_common::source_map::SourceMapGenConfig;
use swc_common::{self, sync::Lrc, FileName, SourceMap};
use swc_core::common::GLOBALS;
use swc_ecma_ast::Module;
use swc_ecma_codegen::Emitter;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};
use swc_ecma_visit::VisitWith;

mod bindings;
mod importer;
mod locate;
mod snippets;
mod transformer;

use importer::Importer;
use transformer::Transformer;

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

impl Preprocessor {
    pub fn new() -> Self {
        Self {
            source_map: Default::default(),
            comments: SingleThreadedComments::default(),
        }
    }

    pub fn parse(
        &self,
        src: &str,
        options: Options,
    ) -> Result<Vec<locate::Occurrence>, swc_ecma_parser::error::Error> {
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
            let parsed_module = parser.parse_module()?;

            let mut visitor = locate::LocateContentTagVisitor::default();

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

            // Prepare to insert the import statement later if needed. This
            // expects a "clean" AST so needs to be run right after parsing.
            //
            // This step also gives us the identifier to use in the transform.
            // Typically, it'd be `template(...)`, but if the existing code
            // already imported the function under a different name, we'll
            // stick with it.
            let import = Importer::prepare(&mut parsed_module, target_module, target_specifier);

            // Run the actual code that transforms the `<template>` syntax into
            // `template(...)`.
            let mut transformer = Transformer::new(import.id());
            transformer.transform(&mut parsed_module);

            // If the transform didn't find any occurrences, then we did not do
            // anything, and do not need to do anything else. (Do we even need
            // source map in that case?)
            if transformer.has_template_tags {
                // If we did find `<template>` tags, we will need to insert the
                // import statement for it, unless the existing code already
                // have it imported for some reason
                import.insert(&mut parsed_module);
            }

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
