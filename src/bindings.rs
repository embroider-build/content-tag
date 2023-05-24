use crate::Preprocessor as CorePreprocessor;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = Error)]
    fn js_error(message: JsValue) -> JsValue;
}

// TODO:
//   - report errors through the bindgen
//   - change the implementation to match my draft rfc
//   - maybe offer a direct file-reading version of the API to avoid the inbound copy
//   - and even when passing a string, see if we can constructor StringInput from JsString

#[wasm_bindgen]
pub struct Preprocessor {
    core: Box<CorePreprocessor>,
}

#[wasm_bindgen]
impl Preprocessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            core: Box::new(CorePreprocessor::new()),
        }
    }

    pub fn preprocess(&self, src: String) -> Result<String, JsValue> {
      self.core.process(src, Default::default())
        .map_err(|_| js_error("Something went wrong".into()))
    }
}
