import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

if (!process.env.VITE_OIDC_ISSUER_URL)
  throw new Error("VITE_OIDC_ISSUER_URL manquant");
if (!process.env.VITE_APP_URL) throw new Error("VITE_APP_URL manquant");
if (!process.env.VITE_BACKEND_URL) throw new Error("VITE_BACKEND_URL manquant");
if (!process.env.VITE_OIDC_REDIRECT_URI)
  throw new Error("VITE_OIDC_REDIRECT_URI manquant");

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      "/api": process.env.VITE_BACKEND_URL,
    },
  },
  test: {
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    globals: true,
  },
});
