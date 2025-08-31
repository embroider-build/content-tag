# content-tag

`content-tag` is a preprocessor for JS files that are using the content-tag proposal. This originated with Ember.js' GJS and GTS functionality. You can read more by [checking out the original RFC](https://rfcs.emberjs.com/id/0931-template-compiler-api/)

This preprocessor can be used to transform files using the `content-tag` spec to standard JS. It is built on top of [swc](https://swc.rs/) using Rust and is deployed as a wasm package.

## Installation

```sh
npm install content-tag
```

## Usage

### Node (CommonJS)

```js
let { Preprocessor } = require("content-tag");
let p = new Preprocessor();
let output = p.process("<template>Hi</template>");

console.log(output);
```

### Node (ESM)

```js
import { Preprocessor } from "content-tag";
let p = new Preprocessor();
let output = p.process("<template>Hi</template>");

console.log(output);
```

### Browser (ESM)

```js
import { Preprocessor } from "content-tag";
let p = new Preprocessor();
let output = p.process("<template>Hi</template>");

console.log(output);
```

## API

### `Preprocessor`

All `content-tag` public API lives on the `Preprocessor` object.

### `Preprocessor.process(src: string, options?: PreprocessorOptions): { code: string, map: string };`

Parses a given source code string using the `content-tag` spec into standard
JavaScript.

```ts
import { Preprocessor } from "content-tag";
let p = new Preprocessor();
let output = p.process("<template>Hi</template>");
```

### `Preprocessor.parse(src: string, options?: PreprocessorOptions): Parsed[];`

Parses a given source code string using the `content-tag` spec into an array of
`Parsed` content tag objects.

```ts
import { Preprocessor } from "content-tag";
let p = new Preprocessor();
let output = p.parse("<template>Hi</template>");
```

#### `PreprocessorOptions`

```ts
interface PreprocessorOptions {
  /** Default is `false` */
  inline_source_map?: boolean;

  filename?: string;
}
```

#### `Parsed`

````ts
interface Range {
  // Range in raw bytes.
  startByte: number;
  endByte: number;

  // Range in unicode codepoints.
  // CAUTION: see "Unicode Codepoint Slicing Warning" below.
  startChar: number;
  endChar: number;

  // utf16 is used by JavaScript strings.
  // e.g. str.slice(range.startUtf16Codepoint, range.endUtf16Codepoint)
  startUtf16Codepoint: number;
  endUtf16Codepoint: number;
}

interface Parsed {
  /**
   * The type for the content tag.
   *
   * 'expression' corresponds to a tag in an expression position, e.g.
   * ```
   * const HiComponent = <template>Hi</template>;
   * ```
   *
   * 'class-member' corresponds to a tag in a class-member position, e.g.
   * ```
   * export default class HiComponent extends Component {
   *   <template>Hi</template>
   * }
   * ```
   */
  type: "expression" | "class-member";

  /**
   * Currently, only template tags are parsed.
   */
  tagName: "template";

  /** Raw template contents. */
  contents: string;

  /**
   * Range of the contents, inclusive of the
   * `<template></template>` tags.
   */
  range: Range;

  /**
   * Range of the template contents, not inclusive of the
   * `<template></template>` tags.
   */
  contentRange: {
    start: number;
    end: number;
  };

  /**
   * Range of the opening `<template>` tag.
   */
  startRange: Range;

  /**
   * Range of the closing `</template>` tag.
   */
  endRange: Range;
}
````

## Unicode Codepoint Slicing Warning

If you have a string and want to use the range provided by our `parse` method to slice out parts of that string, you need avoid two major pitfalls.

First, you want to use our `startChar` and `endChar` not `startByte` and `endByte`. Earlier versions of this library only provided `start` and `end` and they were always bytes, making string slicing unnecessarily difficult.

Second, beware that Javascript's `String.prototype.slice` doesn't actually work on Unicode codepoints. It works on UTF-16 units, which are not the same thing. Intead, you can rely on `String.prototype[Symbol.iterator]` which _does_ work on Unicode codepoints. So this is safe, even when fancy things like emojis are present:

```js
Array.from(myString).slice(range.startChar, range.endChar).join("");
```

if you want to just slice from a string you can use 
```js
str.slice(range.startUtf16Codepoint, range.endUtf16Codepoint)
```

## Contributing

See the [CONTRIBUTING.md](./CONTRIBUTING.md) file.
