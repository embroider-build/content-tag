# glimmer-swc

## Prerequisites

 - rust nightly via rustup (at time of writing I'm using rustc 1.71.0-nightly (f5559e338 2023-04-24))
 - wasm-pack
 - rustup target add wasm32-unknown-unknown

## Running

CLI preprocessor:

`cargo run ./sample/component.gjs`

Tests:

`cargo test`:

Build wasm package:

`wasm-pack build --target=nodejs`

See node package in action:

```sh
node
require('./pkg')
```