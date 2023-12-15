/*
 * wasm-pack doesn't give us correct enough types.
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


/**
*/
export class Preprocessor {
  free(): void;
/**
*/
  constructor();
/**
* @param {string} src
* @param {string | undefined} filename
* @returns {string}
*/
  process(src: string, filename?: string): string;
/**
* @param {string} src
* @param {string | undefined} filename
* @returns {any}
*/
  parse(src: string, filename?: string): Parsed[];
}
