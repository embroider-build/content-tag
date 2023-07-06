# content-tag

`content-tag` is a preprocessor for JS files that are using the content-tag proposal. This originated with Ember.js' GJS and GTS functionality. You can read more by [checking out the original RFC](https://rfcs.emberjs.com/id/0931-template-compiler-api/)

This preprocessor can be used to transform files using the `content-tag` spec to standard JS. It is built on top of [swc](https://swc.rs/) using Rust and is deployed as a wasm package.

## Installation

```sh
# note this will change when we decide where this package lives
npm install @real_ate/content-tag
```

## Usage

```js
let { Preprocessor } = require('content-tag');
let p = new Preprocessor();
let output = p.process('<template>Hi</template>');
console.log(output);
```

## Contributing

See the [CONTRIBUTING.md](./CONTRIBUTING.md) file.

