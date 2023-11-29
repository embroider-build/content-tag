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

    #[wasm_bindgen(js_namespace = JSON, js_name = parse)]
    fn json_parse(value: JsValue) -> JsValue;
}

#[wasm_bindgen]
pub struct Preprocessor {
    // TODO: reusing this between calls result in incorrect spans; there may
    // be value in reusing some part of the stack but we will have to figure
    // out how to combine the APIs correctly to ensure we are not hanging on
    // to the states unexpectedly
    // core: Box<CorePreprocessor>,
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
        // TODO: investigate reuse
        // Self {
        //     core: Box::new(CorePreprocessor::new()),
        // }
        Self {}
    }

    pub fn process(&self, src: String, filename: Option<String>) -> Result<String, JsValue> {
        let preprocessor = CorePreprocessor::new();
        let result = preprocessor.process(
            &src,
            Options {
                filename: filename.map(|f| f.into()),
                inline_source_map: false,
            },
        );

        match result {
            Ok(output) => Ok(output),
            Err(err) => Err(as_javascript_error(err, preprocessor.source_map()).into()),
        }
    }

    pub fn parse(&self, src: String, filename: Option<String>) -> Result<JsValue, JsValue> {
        let preprocessor = CorePreprocessor::new();
        let result = preprocessor
            .parse(
                &src,
                Options {
                    filename: filename.as_ref().map(|f| f.into()),
                    inline_source_map: false,
                },
            )
            .map_err(|_err| self.process(src, filename).unwrap_err())?;
        let serialized = serde_json::to_string(&result)
            .map_err(|err| js_error(format!("Unexpected serialization error; please open an issue with the following debug info: {err:#?}").into()))?;
        Ok(json_parse(serialized.into()))
    }

    pub fn ast(&self, src: String, filename: Option<String>) -> Result<JsValue, JsValue> {
            let preprocessor = CorePreprocessor::new();
            let result = preprocessor
                .ast(
                    &src,
                    Options {
                        filename: filename.as_ref().map(|f| f.into()),
                        inline_source_map: false,
                    },
                )
                .map_err(|_err| self.process(src, filename).unwrap_err())?;
            let serialized = serde_json::to_string(&result)
                .map_err(|err| js_error(format!("Unexpected serialization error; please open an issue with the following debug info: {err:#?}").into()))?;
            Ok(json_parse(serialized.into()))
        }
}
