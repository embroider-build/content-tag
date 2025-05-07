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

### `Preprocessor.process(src: string, options?: PreprocessorOptions): string;`

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

  // Range in unicode characters. If you're trying to slice out parts of the tring, you want this, not the byte.
  //
  // CAUTION: Javascript String.prototype.slice is not actually safe to use on these values
  // because it gets characters beyond UTF16 wrong. You want:
  //     Array.from(myString).slice(startChar, endChar).join('')
  // instead.
  startChar: number;
  endChar: number;
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

## Contributing

See the [CONTRIBUTING.md](./CONTRIBUTING.md) file.
