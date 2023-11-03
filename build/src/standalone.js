export * from "./content-tag.js";

import init, { Preprocessor } from "./content-tag.js";

export async function createPreprocessor() {
  // This no-ops if it's already ran
  await init();

  let processor = new Preprocessor();

  return processor;
}
