use std::path::PathBuf;
use swc_common::{
    self,
    errors::{ColorConfig, Handler},
    sync::Lrc,
    FileName, SourceMap, comments::Comments,
};
use swc_ecma_ast::Module;
use swc_ecma_codegen::Emitter;
use swc_ecma_parser::{lexer::Lexer, EsConfig, Parser, StringInput, Syntax};
use swc_ecma_visit::{as_folder, FoldWith};
use swc_common::comments::SingleThreadedComments;
use wasm_bindgen::prelude::*;

mod transform;

#[derive(Default)]
pub struct Options {
    pub filename: Option<PathBuf>,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = Error)]
    fn js_error(message: JsValue) -> JsValue;
}

#[wasm_bindgen]
pub fn wip_binding(src: String) -> Result<String, JsValue> {
    gjs_to_js(src, Default::default()).map_err(|_| js_error("Something went wrong".into()))
}

pub fn gjs_to_js(src: String, options: Options) -> Result<String, ()> {
    let filename = options.filename.unwrap_or_else(|| "anonymous".into());
    let source_map: Lrc<SourceMap> = Default::default();
    let source_file = source_map.new_source_file(FileName::Real(filename), src);
    let comments = SingleThreadedComments::default();
    let lexer = Lexer::new(
        Syntax::Es(EsConfig {
            decorators: true,
            ..Default::default()
        }),
        Default::default(),
        StringInput::from(&*source_file),
        Some(&comments),
    );
    let mut p = Parser::new_from(lexer);
    let handler =
        Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(source_map.clone()));
    let res = p
        .parse_module()
        .map_err(|e| e.into_diagnostic(&handler).emit());

    for err in p.take_errors() {
        err.into_diagnostic(&handler).emit();
    }

    res.map(|m| {
        let mut tr = as_folder(transform::TransformVisitor);
        let mt = m.fold_with(&mut tr);
        print(&mt, source_map, Some(&comments))
    })
}

fn print(module: &Module, cm: Lrc<SourceMap>, comments: Option<&dyn Comments>) -> String {
    let mut buf = vec![];
    let mut emitter = Emitter {
        cfg: Default::default(),
        cm: cm.clone(),
        wr: Box::new(swc_ecma_codegen::text_writer::JsWriter::new(
            cm.clone(),
            "\n",
            &mut buf,
            None,
        )),
        comments: comments,
    };
    emitter.emit_module(module).unwrap();
    let s = String::from_utf8_lossy(&buf);
    s.to_string()
}
