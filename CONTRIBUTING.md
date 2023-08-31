# Contributing

## Prerequisites

- You need to have rustup installed, and when you run `cargo build` or `cargo test` it will use the nightly rust version that is defined in `rust-toolchain` in the root of the repo.
- You need to run `rustup target add wasm32-unknown-unknown` to download this target for your system
- You need to install wasm-pack https://rustwasm.github.io/wasm-pack/

## Running

CLI:

`cargo run ./sample/component.gjs`

Tests:

`cargo test`:

Build wasm package:

`wasm-pack build --target=nodejs`

which will output your wasm package in `./pkg`

## Running against a local copy of SWC

In Cargo.toml, replace all of `git = "https://github.com/ef4/swc.git", branch = "content-tag"` with `path = "../swc/crates/the_crate_name`.

In VSCode, you can search:

```
([a-z_]+) = \{ git = "https://github.com/ef4/swc.git", branch = "content-tag"
```

and replace:

```
$1 = { path = "../swc/crates/$1"
```