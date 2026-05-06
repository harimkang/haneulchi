import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    globals: true,
    include: ["src/**/*.test.{ts,tsx}", "tests/**/*.test.{ts,tsx}"],
    exclude: ["reference/**", "docs/**", "node_modules/**", "dist/**", "src-tauri/**"],
    setupFiles: "./src/test/setup.ts",
    testTimeout: 60000,
  },
});
