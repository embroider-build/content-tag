import { defineConfig } from "vite";
/**
 * Browser-only vitest config.
 * We're currently using Mocha for node testing.
 */
export default defineConfig({
  test: {
    include: ["test/browser/**/*"],
    browser: {
      name: "chrome",
      headless: true,
      provider: "webdriverio",
      providerOptions: {
        launch: {
          devtools: false,
        },
      },
    },
  },
});
