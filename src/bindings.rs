use crate::{Options, Preprocessor as CorePreprocessor};
use js_sys::Reflect;
use std::path::PathBuf;
use swc_common::{
    errors::{ColorConfig, Handler},
    sync::Lrc,
    SourceMap, Spanned,
};
use swc_error_reporters::{
    handler::{HandlerOpts, ThreadSafetyDiagnostics},
    ErrorEmitter,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = Error)]
    fn js_error(message: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = JSON, js_name = parse)]
    fn json_parse(value: JsValue) -> JsValue;

    #[wasm_bindgen(js_name = Boolean)]
    fn js_boolean(value: &JsValue) -> bool;

    #[wasm_bindgen(js_name = String)]
    fn js_string(value: &JsValue) -> String;
}

impl Options {
    pub fn new(options: JsValue) -> Self {
        if js_boolean(&options) {
            // unwrapping here beacuse we already checked truthiness of
            // `options`, so the normal case of not passing any options has been
            // handled and this will only fail in unusual cases (like a
            // Javascript getter throwing)
            let option_filename = Reflect::get(&options, &"filename".into()).unwrap();
            let filename = if js_boolean(&option_filename) {
                Some(PathBuf::from(js_string(&option_filename)))
            } else {
                None
            };

            Self {
                // unwrap is justified here for the same reasons as commented above
                inline_source_map: js_boolean(
                    &Reflect::get(&options, &"inline_source_map".into()).unwrap(),
                ),
                filename,
            }
        } else {
            Self {
                inline_source_map: false,
                filename: None,
            }
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct CodeMapPair {
    pub code: String,
    pub map: String,
}

#[wasm_bindgen]
impl CodeMapPair {
    #[wasm_bindgen(constructor)]
    pub fn new(code: String, map: String) -> Self {
        Self { code, map }
    }
}

#[wasm_bindgen]
pub struct Preprocessor {
    // TODO: reusing this between calls result in incorrect spans; there may
    // be value in reusing some part of the stack but we will have to figure
    // out how to combine the APIs correctly to ensure we are not hanging on
    // to the states unexpectedly
    // core: Box<CorePreprocessor>,
}

fn capture_err_detail(
    err: swc_ecma_parser::error::Error,
    source_map: Lrc<SourceMap>,
    color: ColorConfig,
) -> JsValue {
    let diagnostics = ThreadSafetyDiagnostics::default();
    let emitter = ErrorEmitter {
        diagnostics: diagnostics.clone(),
        cm: source_map.clone(),
        opts: HandlerOpts {
            color,
            skip_filename: false,
        },
    };
    let handler = Handler::with_emitter(true, false, Box::new(emitter));
    err.into_diagnostic(&handler).emit();
    diagnostics
        .to_pretty_string(&source_map, false, color)
        .join("")
        .into()
}

fn as_javascript_error(err: swc_ecma_parser::error::Error, source_map: Lrc<SourceMap>) -> JsValue {
    let short_desc = format!("Parse Error at {}", source_map.span_to_string(err.span()));
    let js_err = js_error(short_desc.into());
    js_sys::Reflect::set(
        &js_err,
        &"source_code".into(),
        &capture_err_detail(err.clone(), source_map.clone(), ColorConfig::Never),
    )
    .unwrap();
    js_sys::Reflect::set(
        &js_err,
        &"source_code_color".into(),
        &capture_err_detail(err, source_map, ColorConfig::Always),
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

    pub fn process(&self, src: String, options: JsValue) -> Result<CodeMapPair, JsValue> {
        let options = Options::new(options);
        let preprocessor = CorePreprocessor::new();
        let result = preprocessor.process(&src, options);

        match result {
            Ok(output) => Ok(CodeMapPair::new(output.code, output.map)),
            Err(err) => Err(as_javascript_error(err, preprocessor.source_map()).into()),
        }
    }

    pub fn parse(&self, src: String, options: JsValue) -> Result<JsValue, JsValue> {
        let options = Options::new(options);
        let preprocessor = CorePreprocessor::new();
        let result = preprocessor.parse(&src, options);

        match result {
            Ok(parsed) => {
                match serde_json::to_string(&parsed) {
                    Ok(serialized) => Ok(json_parse(serialized.into())),
                    Err(err) =>  Err(js_error(format!("Unexpected serialization error; please open an issue with the following debug info: {err:#?}").into()))
                }
            },
            Err(err) => Err(as_javascript_error(err, preprocessor.source_map()))
        }
    }
}
