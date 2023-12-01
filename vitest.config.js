import { defineConfig, mergeConfig } from "vitest/config";
import viteConfig from "./vite.config.js";

export default defineConfig({
  resolve: {
    alias: {
      // https://github.com/vitejs/vite/issues/9731
      "content-tag": ".",
    },
  },
  test: {
    exclude: ["test/*", "sample/*", "src/*"],
    include: ["test-browser/*"],
    open: false,
    browser: {
      enabled: true,
      name: "chrome",
    },
  },
});
