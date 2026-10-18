export interface Env {
  appLang: string;
  appUrl: string;
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

  // Optional — UI language, defaults to "en" (see i18n.ts).
  const appLangRaw = (get("APP_LANG") ?? "").trim();
  const appLang = appLangRaw || "en";

  const appUrl = take("APP_URL");
  const oidcIssuerUrl = take("OIDC_ISSUER_URL");
  const oidcRedirectUri = take("OIDC_REDIRECT_URI");

  if (missing.length) {
    throw new Error(`missing or empty env vars: ${missing.join(", ")}`);
  }

  return { appLang, appUrl, oidcIssuerUrl, oidcRedirectUri };
}

export const env: Env = createEnv();

// Backward-compatible named exports — single source is `env`.
export const APP_URL = env.appUrl;
export const OIDC_ISSUER_URL = env.oidcIssuerUrl;
