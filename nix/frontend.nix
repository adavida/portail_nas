# Frontend with OIDC URLs baked at build time (import.meta.env) — overridden
# by the NixOS module via overrideAttrs.
{ pkgs }:
{
  appUrl ? "http://localhost:5173",
  oidcIssuerUrl ? "https://127.0.0.1:9091",
  oidcRedirectUri ? "http://localhost:5173/callback",
}:

pkgs.buildNpmPackage {
  pname = "portail-frontend";
  version = "0.1.0";
  src = ../frontend;
  npmDepsHash = "sha256-m7ZXnXFBVNeVI4jlhAjr5hRznOoHdMMTT5xugrkDZVI=";
  VITE_APP_URL = appUrl;
  VITE_BACKEND_URL = appUrl;
  VITE_OIDC_ISSUER_URL = oidcIssuerUrl;
  VITE_OIDC_REDIRECT_URI = oidcRedirectUri;
  # Vite app: serve the static `dist` build, not the npm package stub.
  installPhase = ''
    runHook preInstall
    mkdir -p $out
    cp -r dist/. $out/
    runHook postInstall
  '';
}
