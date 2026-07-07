import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// `base: "./"` -> relative asset URLs (served from elyra://localhost/).
export default defineConfig({
  plugins: [svelte()],
  base: "./",
  build: { outDir: "dist", emptyOutDir: true, target: "esnext" },
  server: { port: 5173, strictPort: true },
});
