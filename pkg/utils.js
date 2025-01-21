import { Preprocessor } from 'content-tag';

let p = new Preprocessor();

export function transformEachContents(source, callback) {

  // Will be added to or subtracted from depending on the change
  // in character length each of the transforms
  let offset = 0;
  let result = source;
  let originalSource = source;
  let buffer = Buffer.from(originalSource, 'utf8');

  let templates = p.parse(source);

  for (let parsed of templates) {
    let transformed = callback(parsed.content)

    /**
     * NOTE: range.start w/ range.end is between the <template> and </template>
     */
    let originalContent = buffer.slice(parsed.range.start, parsed.range.end).toString();
    let originalLength = originalContent.length;
    let originalBeforeContent = buffer.slice(0, parsed.range.start).toString();
    let originalStart = originalBeforeContent.length;
    let originalEnd = originalStart + originalLength;

    /**
     * Need to make sure the opening <template> and closing </template>
     * are not removed.
     *
     * We aren't just using the strings <template> and </template>, because
     * its possible for the opening <template ..... > to have attributes in the future
     * with futher syntax extensions
     * - Signature
     * - defaults?
     * - macros?
    */
    let openingTag = buffer.slice(parsed.startRange.start, parsed.startRange.end).toString();
    let closingTag = buffer.slice(parsed.endRange.start, parsed.endRange.end).toString();

    result =
      result.slice(0, originalStart + offset) +
      openingTag +
      transformed +
      closingTag +
      result.slice(originalEnd + offset, result.length);

    offset = transformed.length - originalLength;
  }

  return result;
}

export function coordinatesOf(source, parsedResult) {
  /**
   * range is the full range, including the leading and trailing <tempalte>,</template>
   * contentRange is the range between / excluding the leading and trailing <template>,</template>
   */
  let { contentRange: byteRange } = parsedResult;
  let buffer = Buffer.from(source, 'utf8');
  let inclusiveContent = buffer.slice(byteRange.start, byteRange.end).toString();
  let beforeContent = buffer.slice(0, byteRange.start).toString();
  let before = beforeContent.length;

  let startCharIndex = before;
  let endCharIndex = before + inclusiveContent.length;

  const contentBeforeTemplateStart = beforeContent.split('\n');
  const lineBeforeTemplateStart = contentBeforeTemplateStart.at(-1);

  /**
   * Reminder:
   *   Rows are 1-indexed
   *   Columns are 0-indexed
   *
   * (for when someone inevitably needs to debug this and is comparing
   *  with their editor (editors typically use 1-indexed columns))
   */
  return {
    line: contentBeforeTemplateStart.length,
    column: lineBeforeTemplateStart.length,
    // character index, not byte index
    start: startCharIndex,
    // character index, not byte index
    end: endCharIndex,
    // any indentation of the <template> parts (class indentation etc)
    columnOffset: lineBeforeTemplateStart.length - lineBeforeTemplateStart.trimStart().length,
  };
}

export function reverseInnerCoordinates(templateCoordinates, innerCoordinates) {
  /**
   * Given the sample source code:
   * 1 export class SomeComponent extends Component<Args> {\n
   * 2     <template>\n
   * 3         {{debugger}}\n
   * 4     </template>\n
   * 5 }
   *
   * The extracted template will be:
   * 1 \n
   * 2    {{debugger}}\n
   *
   * The coordinates of the template in the source file are: { line: 3, column: 14 }.
   * The coordinates of the error in the template are: { line: 2, column: 4 }.
   *
   * Thus, we need to always subtract one before adding the template location.
   */
  const line = innerCoordinates.line + templateCoordinates.line - 1;
  const endLine = innerCoordinates.endLine + templateCoordinates.line - 1;

  /**
   * Given the sample source code:
   * 1 export class SomeComponent extends Component<Args> {\n
   * 2     <template>{{debugger}}\n
   * 3     </template>\n
   * 4 }
   *
   * The extracted template will be:
   * 1 {{debugger}}\n
   *
   * The coordinates of the template in the source file are: { line: 3, column: 14 }.
   * The coordinates of the error in the template are: { line: 1, column: 0 }.
   *
   * Thus, if the error is found on the first line of a template,
   * then we need to add the column location to the result column location.
   *
   * Any result > line 1 will not require any column correction.
   */
  const column = innerCoordinates.line === 1 ? innerCoordinates.column + templateCoordinates.column : innerCoordinates.column;
  const endColumn = innerCoordinates.line === 1 ? innerCoordinates.endColumn + templateCoordinates.column : innerCoordinates.endColumn;

  return {
    line,
    endLine,
    column,
    endColumn,
  };
}
