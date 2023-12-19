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

export interface ParseError {
  /**
   * Formatted output for CLI
   */
  source_code: string;
  /**
   * Color-Formatted output for CLI
   */
  source_code_color: string;

  /**
   * 0-indexed starting line of the error
   */
  start_line: number;
  /**
   * 0-indexed starting character-based column of the error
   */
  start_column: number;
  /**
   * 0-indexed ending line of the error
   */
  end_line: number;
  /**
   * 0-indexed ending character-based column of the error
   */
  end_column: number;
}
