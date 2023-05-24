use std::path::PathBuf;

use swc_common::{
    self,
    errors::{ColorConfig, Handler},
    sync::Lrc,
    FileName, SourceMap,
};
use swc_ecma_ast::Module;
use swc_ecma_codegen::Emitter;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, EsConfig};
use swc_ecma_visit::{as_folder, FoldWith};

mod transform;

#[derive(Default)]
pub struct Options {
    pub filename: Option<PathBuf>,
}

pub fn gjs_to_js(src: String, options: Options) -> Result<String, ()> {
    let filename = options.filename.unwrap_or_else(|| "anonymous".into());
    let source_map: Lrc<SourceMap> = Default::default();
    let source_file = source_map.new_source_file(FileName::Real(filename), src);
    let lexer = Lexer::new(
        Syntax::Es(EsConfig { decorators: true , ..Default::default() }),
        Default::default(),
        StringInput::from(&*source_file),
        None,
    );
    let mut p = Parser::new_from(lexer);
    let handler =
        Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(source_map.clone()));
    let res = p
        .parse_module()
        .map_err(|e| e.into_diagnostic(&handler).emit());

    res.map(|m| {
        let mut tr = as_folder(transform::TransformVisitor);
        let mt = m.fold_with(&mut tr);
        print(&mt, source_map)
    })
}

fn print(module: &Module, cm: Lrc<SourceMap>) -> String {
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
        comments: None,
    };
    emitter.emit_module(module).unwrap();
    let s = String::from_utf8_lossy(&buf);
    s.to_string()
}
