use swc_common::comments::SingleThreadedComments;
use swc_common::{self, sync::Lrc, FileName, SourceMap};
use swc_ecma_ast::{Expr, ExprOrSpread};
use swc_ecma_parser::{lexer::Lexer, EsConfig, Parser, StringInput, Syntax};

pub fn parse_expression(src: &str) -> Box<Expr> {
    let filename = "glimmer-template-prelude.js".into();
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
