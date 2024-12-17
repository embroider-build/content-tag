/**
 * Not all our entrypoints support all environments.
 *
 * This stub file is to appears are-the-types-wrong, but also give developers in require / CJS environments
 * more useful information when they try to require an ESM
 */
class Preprocessor {
  constructor() {
    throw new Error(`Tried to create a Preprocessor using require() when an import was expected. Please change to using import.`);
  }
}


module.exports = { Preprocessor };
