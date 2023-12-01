import { defineConfig, mergeConfig } from "vitest/config";

export default defineConfig({
  test: {
    browser: {
      enabled: true,
      name: "chrome",
    },
  },
});
