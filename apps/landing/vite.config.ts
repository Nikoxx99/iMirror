import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  base: process.env.GITHUB_ACTIONS ? "/iMirror/" : "/",
  plugins: [react()],
  server: {
    host: "0.0.0.0",
    port: 5175
  }
});
