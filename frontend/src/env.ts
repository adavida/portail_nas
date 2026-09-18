function requireEnv(name: string, value: string | undefined): string {
  if (!value)
    throw new Error(
      `${name} missing: set ${name} in devenv.nix env (VITE_${name})`,
    );
  return value;
}

export const APP_URL = requireEnv(
  "APP_URL",
  import.meta.env.VITE_APP_URL as string | undefined,
);
export const BACKEND_URL = requireEnv(
  "BACKEND_URL",
  import.meta.env.VITE_BACKEND_URL as string | undefined,
);
export const OIDC_ISSUER_URL = requireEnv(
  "OIDC_ISSUER_URL",
  import.meta.env.VITE_OIDC_ISSUER_URL as string | undefined,
);
export const OIDC_REDIRECT_URI = requireEnv(
  "OIDC_REDIRECT_URI",
  import.meta.env.VITE_OIDC_REDIRECT_URI as string | undefined,
);

export function appUrl(): string {
  return APP_URL;
}
export function backendUrl(): string {
  return BACKEND_URL;
}
export function oidcIssuerUrl(): string {
  return OIDC_ISSUER_URL;
}
export function oidcRedirectUri(): string {
  return OIDC_REDIRECT_URI;
}
