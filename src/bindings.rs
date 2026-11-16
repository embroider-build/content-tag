use crate::{Options, Preprocessor as CorePreprocessor};
use js_sys::Reflect;
use std::path::PathBuf;
use swc_common::{errors::Handler, sync::Lrc, SourceMap, Spanned};
use swc_error_reporters::{
    handler::{HandlerOpts, ThreadSafetyDiagnostics},
    ErrorEmitter, GraphicalReportHandler, GraphicalTheme, ToPrettyDiagnostic,
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

#[wasm_bindgen]
pub struct Preprocessor {
    core: CorePreprocessor,
}

fn capture_err_detail(
    err: swc_ecma_parser::error::Error,
    source_map: Lrc<SourceMap>,
    theme: GraphicalTheme,
) -> JsValue {
    let mut diagnostics = ThreadSafetyDiagnostics::default();
    let emitter = ErrorEmitter {
        diagnostics: diagnostics.clone(),
        cm: source_map.clone(),
        opts: HandlerOpts::default(),
    };
    let handler = Handler::with_emitter(true, false, Box::new(emitter));
    err.into_diagnostic(&handler).emit();
    let reporter = GraphicalReportHandler::new_themed(theme);
    let s: String = diagnostics
        .take()
        .iter()
        .map(|d| d.to_pretty_string(&source_map, false, &reporter))
        .collect();
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
            core: CorePreprocessor::new(),
        }
    }

    /// Returns a plain `{ code, map }` object. The strings are copied into
    /// JavaScript, so nothing is left in wasm memory waiting for the
    /// JavaScript object to be garbage collected.
    pub fn process(&self, src: String, options: JsValue) -> Result<JsValue, JsValue> {
        let options = Options::new(options);
        let result = self.core.process(&src, options);

        match result {
            Ok(output) => {
                let result = js_sys::Object::new();
                Reflect::set(&result, &"code".into(), &output.code.into())?;
                Reflect::set(&result, &"map".into(), &output.map.into())?;
                Ok(result.into())
            }
            Err(err) => Err(as_javascript_error(err.error, err.source_map)),
        }
    }

    pub fn parse(&self, src: String, options: JsValue) -> Result<JsValue, JsValue> {
        let options = Options::new(options);
        let result = self.core.parse(&src, options);

        match result {
            Ok(parsed) => {
                match serde_json::to_string(&parsed) {
                    Ok(serialized) => Ok(json_parse(serialized.into())),
                    Err(err) =>  Err(js_error(format!("Unexpected serialization error; please open an issue with the following debug info: {err:#?}").into()))
                }
            },
            Err(err) => Err(as_javascript_error(err.error, err.source_map))
        }
    }
}
