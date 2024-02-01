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
let { Preprocessor } = require('content-tag');
let p = new Preprocessor();
let output = p.process('<template>Hi</template>');

console.log(output);
```


### Node (ESM)

```js
import { Preprocessor } from 'content-tag';
let p = new Preprocessor();
let output = p.process('<template>Hi</template>');

console.log(output);
```

### Browser (ESM)

```js
import { Preprocessor } from 'content-tag';
let p = new Preprocessor();
let output = p.process('<template>Hi</template>');

console.log(output);
```

## API

### `Preprocessor`

All `content-tag` public API lives on the `Preprocessor` object.

### `Preprocessor.process(src: string, options?: PreprocessorOptions): string;`

Parses a given source code string using the `content-tag` spec into standard
JavaScript.

```ts
import { Preprocessor } from 'content-tag';
let p = new Preprocessor();
let output = p.process('<template>Hi</template>');
```

### `Preprocessor.parse(src: string, options?: PreprocessorOptions): Parsed[];`

Parses a given source code string using the `content-tag` spec into an array of
`Parsed` content tag objects.

```ts
import { Preprocessor } from 'content-tag';
let p = new Preprocessor();
let output = p.parse('<template>Hi</template>');
```

#### `PreprocessorOptions`

````ts
interface PreprocessorOptions {

  /** Default is `false` */
  inline_source_map?: boolean;

  filename?: string;

}
````

#### `Parsed`

NOTE: All ranges are in bytes, not characters.

````ts
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
   * Byte range of the contents, inclusive of inclusive of the
   * `<template></template>` tags.
   */
  range: {
    start: number;
    end: number;
  };

  /**
   * Byte range of the template contents, not inclusive of the
   * `<template></template>` tags.
   */
  contentRange: {
    start: number;
    end: number;
  };

  /** Byte range of the opening `<template>` tag. */
  startRange: {
    end: number;
    start: number;
  };

  /** Byte range of the closing `</template>` tag. */
  endRange: {
    start: number;
    end: number;
  };
}
````

## Contributing

See the [CONTRIBUTING.md](./CONTRIBUTING.md) file.

