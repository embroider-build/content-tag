#!/bin/bash

# we need npm packages for the post-wasm phase
npm install

rm -rf pkg

# wasm-pack knows to use wasm-opt, when present
# NOTE: wasm-pack does not support multi-target building
#       so we'll build twice, and then tweak package.json
#       "exports" to point at the correct build depending on node or browser
wasm-pack build --target web --out-dir pkg/standalone --weak-refs --no-pack --release
wasm-pack build --target nodejs --out-dir pkg/node --weak-refs --no-pack --release

# Rename the node js file to cjs, because we emit type=module
mv pkg/node/content_tag.js pkg/node/content_tag.cjs

# generate the rest of the package for npm, since
# we disabled package.json generation above with --no-pack.
# this needs to be done because we're targeting
# both browser and node, which wasm-packg doesn't natively support.
node ./build/post-wasm-build.mjs

# ---------
cp LICENSE pkg/LICENSE
cp README.md pkg/README.md
