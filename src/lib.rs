use std::path::PathBuf;
use swc_common::comments::SingleThreadedComments;
use swc_common::{self, sync::Lrc, FileName, SourceMap};
use swc_ecma_ast::Module;
use swc_ecma_codegen::Emitter;
use swc_ecma_parser::{lexer::Lexer, EsConfig, Parser, StringInput, Syntax};
use swc_ecma_visit::{as_folder, FoldWith};
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

// Next:
//   - report errors through the bindgen
//   - change the implementation to match my draft rfc
//   - maybe offer a direct file-reading version of the API to avoid the inbound copy
//   - and even when passing a string, see if we can constructor StringInput from JsString

#[wasm_bindgen]
pub fn wip_binding(src: String) -> Result<String, JsValue> {
    let p = Preprocessor::new();
    p.process(src, Default::default())
        .map_err(|_| js_error("Something went wrong".into()))
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
        src: String,
        options: Options,
    ) -> Result<String, swc_ecma_parser::error::Error> {
        let filename = options.filename.unwrap_or_else(|| "anonymous".into());
        let source_file = self
            .source_map
            .new_source_file(FileName::Real(filename), src);
        let lexer = Lexer::new(
            Syntax::Es(EsConfig {
                decorators: true,
                ..Default::default()
            }),
            Default::default(),
            StringInput::from(&*source_file),
            Some(&self.comments),
        );
        let mut p = Parser::new_from(lexer);
        let m = p.parse_module()?;
        let mut tr = as_folder(transform::TransformVisitor);
        let mt = m.fold_with(&mut tr);
        Ok(self.print(&mt))
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
