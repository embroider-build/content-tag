use crate::{Options, Preprocessor as CorePreprocessor};
use std::{fmt, str};
use swc_common::{
    errors::Handler,
    sync::{Lock, Lrc},
    SourceMap, Spanned,
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

fn capture_err_detail(
    err: swc_ecma_parser::error::Error,
    source_map: Lrc<SourceMap>,
    theme: GraphicalTheme,
) -> JsValue {
    let wr = Writer::default();
    let emitter = PrettyEmitter::new(
        source_map,
        Box::new(wr.clone()),
        GraphicalReportHandler::new_themed(theme),
        Default::default(),
    );
    let handler = Handler::with_emitter(true, false, Box::new(emitter));
    err.into_diagnostic(&handler).emit();
    let s = wr.0.lock().as_str().to_string();
    s.into()
}

fn as_javascript_error(err: swc_ecma_parser::error::Error, source_map: Lrc<SourceMap>) -> JsValue {
    let short_desc = format!("Parse Error at {}", source_map.span_to_string(err.span()));
    let js_err = js_error(short_desc.into());
    js_sys::Reflect::set(
        &js_err,
        &"source_code".into(),
        &capture_err_detail(
            err.clone(),
            source_map.clone(),
            GraphicalTheme::unicode_nocolor(),
        ),
    )
    .unwrap();
    js_sys::Reflect::set(
        &js_err,
        &"source_code_color".into(),
        &capture_err_detail(err, source_map, GraphicalTheme::unicode()),
    )
    .unwrap();
    return js_err;
}

#[wasm_bindgen]
impl Preprocessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            core: Box::new(CorePreprocessor::new()),
        }
    }

    pub fn process(&self, src: String, filename: Option<String>) -> Result<String, JsValue> {
        let result = self.core.process(
            &src,
            Options {
                filename: filename.map(|f| f.into()),
                inlineSourcemap: false,
            },
        );

        match result {
            Ok(output) => Ok(output),
            Err(err) => Err(as_javascript_error(err, self.core.source_map()).into()),
        }
    }
}
