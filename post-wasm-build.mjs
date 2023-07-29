// This post-wasm-build.js script is called from build.sh
import fs from 'node:fs/promises';
import path from 'node:path';
import toml from 'toml';

let cargo = await fs.readFile('Cargo.toml', 'utf8');
let config = toml.parse(cargo);


const manifest = {
  "name": config.package.name,
  "description": config.package.description,
  "version": config.package.version,
  "license": config.package.license,
  "repository": {
    "type": "git",
    "url": "https://github.com/embroider-build/content-tag"
  },
  "files": [
    "browser",
    "node"
  ],
  "type": "module",
  "exports": {
    ".": {
      "import": "./browser/content_tag.js",
      "require": "./node/content_tag.cjs",
      "types": "./browser/content_tag.d.ts"
    }
  }
};



const content = JSON.stringify(manifest, null, 2);

await fs.writeFile('pkg/package.json', content);
