/*
 * wasm-pack doesn't give us correct enough types.
 * this file must be manually kept in sync with ./index.d.ts
 */

interface Range {
  startByte: number;
  endByte: number;
  startChar: number;
  endChar: number;
}

interface Parsed {
  type: "expression" | "class-member";
  tagName: "template";
  contents: string;
  range: Range;
  contentRange: Range;
  startRange: Range;
  endRange: Range;
}

interface PreprocessorOptions {
  /** Default is `false` */
  inline_source_map?: boolean;

  filename?: string;
}

/**
 */
export class Preprocessor {
  free(): void;
  /**
   */
  constructor();
  /**
   * @param {string} src
   * @param {PreprocessorOptions | undefined} options
   * @returns {string}
   */
  process(
    src: string,
    options?: PreprocessorOptions
  ): { code: string; map: string };
  /**
   * @param {string} src
   * @param {PreprocessorOptions | undefined} options
   * @returns {any}
   */
  parse(src: string, options?: PreprocessorOptions): Parsed[];
}
