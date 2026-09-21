import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    globals: true,
    exclude: ["node_modules", "dist", "e2e/**"],
    //   setupFiles: "./tests/setup.ts", // Optional: for global jest-dom matchers if needed later
  },
});
