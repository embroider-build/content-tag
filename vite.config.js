import { defineConfig } from "vite";

export default defineConfig({
  resolve: {
    alias: {
      // https://github.com/vitejs/vite/issues/9731
      "content-tag": ".",
    },
  },
});
