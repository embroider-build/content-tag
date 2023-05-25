# glimmer-swc

## Prerequisites

- rust nightly via rustup (at time of writing I'm using rustc 1.71.0-nightly (f5559e338 2023-04-24))
- wasm-pack
- rustup target add wasm32-unknown-unknown
- you need to clone ef4/swc `content-tag` branch and have it located next to your clone of this repo

## Running

CLI:

`cargo run ./sample/component.gjs`

Tests:

`cargo test`:

Build wasm package:

`wasm-pack build --target=nodejs`

See wasm package in action:

```sh
let { Preprocessor } = require('./pkg');
let p = new Preprocessor();
p.process('<template>Hi</template>');
```
