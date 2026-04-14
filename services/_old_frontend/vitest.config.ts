import { resolve } from "node:path";
import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/__tests__/vitest-setup.ts"],
    exclude: [
      "e2e/**",
      "node_modules/**",
      ".next/**",
      "dist/**",
      "**/node_modules/**",
    ],
  },
  resolve: {
    alias: {
      "@": resolve(__dirname, "./src"),
    },
  },
});
