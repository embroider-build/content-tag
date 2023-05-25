
use swc_common::comments::SingleThreadedComments;
use swc_common::{self, sync::Lrc, FileName, SourceMap};
use swc_ecma_codegen::Emitter;
use swc_ecma_parser::{lexer::Lexer, EsConfig, Parser, StringInput, Syntax};

use crate::Preprocessor;

pub fn testcase(input: &str, expected: &str) -> Result<(), swc_ecma_parser::error::Error> {
    let p = Preprocessor::new();
    assert_eq!(p.process(input, Default::default())?, normalize(expected));
    Ok(())
}

fn normalize(src: &str) -> String {
    let filename = "test.js".into();

    let source_map: Lrc<SourceMap> = Default::default();
    let comments: SingleThreadedComments = Default::default();

    let source_file = source_map.new_source_file(FileName::Real(filename), src.to_string());

    let lexer = Lexer::new(
        Syntax::Es(EsConfig {
            decorators: true,
            ..Default::default()
        }),
        Default::default(),
        StringInput::from(&*source_file),
        Some(&comments),
    );
    let mut parser = Parser::new_from(lexer);
    let parsed_module = parser.parse_module().unwrap();
    let mut buf = vec![];
    let mut emitter = Emitter {
        cfg: Default::default(),
        cm: source_map.clone(),
        wr: Box::new(swc_ecma_codegen::text_writer::JsWriter::new(
            source_map.clone(),
            "\n",
            &mut buf,
            None,
        )),
        comments: Some(&comments),
    };
    emitter.emit_module(&parsed_module).unwrap();
    let s = String::from_utf8_lossy(&buf);
    s.to_string()
}
