// This post-wasm-build.js script is called from build.sh
import fs from "node:fs/promises";
import path from "node:path";
import url from "node:url";
import toml from "toml";

let cargo = await fs.readFile("Cargo.toml", "utf8");
let config = toml.parse(cargo);

const manifest = {
  name: config.package.name,
  description: config.package.description,
  version: config.package.version,
  license: config.package.license,
  repository: {
    type: "git",
    url: "https://github.com/embroider-build/content-tag",
  },
  files: ["standalone", "node"],
  type: "module",
  exports: {
    ".": {
      types: "./node/content_tag.d.ts",
      default: "./node/content_tag.cjs",
    },
    "./standalone": {
      types: "./standalone/standalone.d.ts",
      import: "./standalone/standalone.js",
    },
    "./standalone/*": {
      types: "./standalone/*.d.ts",
      import: "./standalone/*.js",
    },
  },
};

const content = JSON.stringify(manifest, null, 2);

const here = url.fileURLToPath(new URL(".", import.meta.url));
const root = path.join(here, "..");
const output = path.join(root, "pkg");

await fs.writeFile(path.join(output, "package.json"), content);
await fs.copyFile(
  path.join(here, "src/standalone.js"),
  path.join(output, "standalone/standalone.js")
);
await fs.copyFile(
  path.join(here, "src/standalone.d.ts"),
  path.join(output, "standalone/standalone.d.ts")
);
