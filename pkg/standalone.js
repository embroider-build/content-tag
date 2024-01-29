import init from "./standalone/content_tag.js";
import { Preprocessor as WasmPreprocessor } from "./standalone/content_tag.js";

await init();

const defaultOptions = {
  inline_source_map: false,
  filename: null
};

export class Preprocessor {
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
