export interface Env {
  appUrl: string;
  backendUrl: string;
  oidcIssuerUrl: string;
  oidcRedirectUri: string;
}

export function createEnv(
  get: (name: string) => string | undefined = (name) =>
    import.meta.env[`VITE_${name}`] as string | undefined,
): Env {
  const missing: string[] = [];
  const take = (name: string): string => {
    const v = get(name);
    if (!v || !v.trim()) {
      missing.push(name);
      return "";
    }
    return v;
  };

  const appUrl = take("APP_URL");
  const backendUrl = take("BACKEND_URL");
  const oidcIssuerUrl = take("OIDC_ISSUER_URL");
  const oidcRedirectUri = take("OIDC_REDIRECT_URI");

  if (missing.length) {
    throw new Error(
      `missing or empty env vars: ${missing.join(", ")} — set them in devenv.nix env (VITE_...)`,
    );
  }

  return { appUrl, backendUrl, oidcIssuerUrl, oidcRedirectUri };
}

export const env: Env = createEnv();

// Backward-compatible named exports — single source is `env`.
export const APP_URL = env.appUrl;
export const BACKEND_URL = env.backendUrl;
export const OIDC_ISSUER_URL = env.oidcIssuerUrl;
export const OIDC_REDIRECT_URI = env.oidcRedirectUri;

export function appUrl(): string {
  return env.appUrl;
}
export function backendUrl(): string {
  return env.backendUrl;
}
export function oidcIssuerUrl(): string {
  return env.oidcIssuerUrl;
}
export function oidcRedirectUri(): string {
  return env.oidcRedirectUri;
}
