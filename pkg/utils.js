import { Preprocessor } from 'content-tag';

export function transform(source, callback)  {
  let p = new Preprocessor(source);

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

