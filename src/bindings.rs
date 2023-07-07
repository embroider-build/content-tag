use crate::{Options, Preprocessor as CorePreprocessor};
use std::{fmt, path::PathBuf, str};
use swc_common::{
    errors::Handler,
    sync::{Lock, Lrc},
    SourceMap,
};
use swc_error_reporters::{GraphicalReportHandler, GraphicalTheme, PrettyEmitter};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = Error)]
    fn js_error(message: JsValue) -> JsValue;
}

#[wasm_bindgen]
pub struct Preprocessor {
    core: Box<CorePreprocessor>,
}

#[derive(Clone, Default)]
struct Writer(Lrc<Lock<String>>);

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.lock().write_str(s)
    }
}

fn err_to_string(err: swc_ecma_parser::error::Error, source_map: Lrc<SourceMap>) -> String {
    let wr = Writer::default();

    let emitter = PrettyEmitter::new(
        source_map,
        Box::new(wr.clone()),
        GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor()),
        Default::default(),
    );

    let handler = Handler::with_emitter(true, false, Box::new(emitter));

    err.into_diagnostic(&handler).emit();

    let s = wr.0.lock().as_str().to_string();
    return s.into();
}

#[wasm_bindgen]
impl Preprocessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            core: Box::new(CorePreprocessor::new()),
        }
    }

    pub fn process(&self, src: String) -> Result<String, JsValue> {
        let result = self.core.process(
            &src,
            Options {
                // when we start passing the filename to the preprocessor we will have a better file name here
                filename: Some(PathBuf::new()),
            },
        );

        match result {
            Ok(output) => Ok(output),
            Err(err) => Err(js_error(err_to_string(err, self.core.source_map()).into())),
        }
    }
}
