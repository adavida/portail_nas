import { svelte } from "@sveltejs/vite-plugin-svelte";
import { svelteTesting } from "@testing-library/svelte/vite";
import { defineConfig } from "vite";

if (!process.env.VITE_OIDC_ISSUER_URL)
  throw new Error("VITE_OIDC_ISSUER_URL manquant");
if (!process.env.VITE_APP_URL) throw new Error("VITE_APP_URL manquant");
if (!process.env.VITE_OIDC_REDIRECT_URI)
  throw new Error("VITE_OIDC_REDIRECT_URI manquant");

export default defineConfig({
  plugins: [svelte(), ...(process.env.VITEST ? [svelteTesting()] : [])],
  css: {
    preprocessorOptions: {
      scss: { api: "modern-compiler" },
    },
  },
  server: {
    port: 5173,
    proxy: {
      "/api": process.env.VITE_BACKEND_URL ?? process.env.VITE_APP_URL,
    },
  },
  test: {
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    globals: true,
  },
});
