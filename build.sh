#!/bin/bash

# we need npm packages for the post-wasm phase
npm install

rm -rf pkg/node
rm -rf pkg/standalone

# wasm-pack knows to use wasm-opt, when present
# NOTE: wasm-pack does not support multi-target building
#       so we'll build twice, and then tweak package.json
#       "exports" to point at the correct build depending on node or browser
wasm-pack build --target web --out-dir pkg/standalone --weak-refs --no-pack --release
wasm-pack build --target nodejs --out-dir pkg/node --weak-refs --no-pack --release

# Rename the node js file to cjs, because we emit type=module
mv pkg/node/content_tag.js pkg/node/content_tag.cjs
