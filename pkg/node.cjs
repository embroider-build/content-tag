const { Preprocessor: WasmPreprocessor } = require("./node/content_tag.cjs");

const defaultOptions = {
  inline_source_map: false,
  filename: null
};

class Preprocessor {
  #preprocessor;

  constructor() {
    this.#preprocessor = new WasmPreprocessor();
  }

  process(str, options = {}) {
    return this.#preprocessor.process(str, { ...defaultOptions, ...options });
  }

  parse(str, options = {}) {
    return this.#preprocessor.parse(str, { ...defaultOptions, ...options });
  }
}

module.exports.Preprocessor = Preprocessor;
