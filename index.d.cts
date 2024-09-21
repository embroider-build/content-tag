/*
 * wasm-pack doesn't give us correct enough types.
* this file must be manually kept in sync with ./index.d.ts
 */



interface Parsed {
  type: 'expression' | 'class-member';
  tagName: 'template';
  contents: string;
  range: {
    start: number;
    end: number;
  };
  contentRange: {
    start: number;
    end: number;
  };
  startRange: {
    end: number;
    start: number;
  };
  endRange: {
    start: number;
    end: number;
  };
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
  process(src: string, options?: PreprocessorOptions): string;
/**
* @param {string} src
* @param {PreprocessorOptions | undefined} options
* @returns {any}
*/
  parse(src: string, options?: PreprocessorOptions): Parsed[];
}
