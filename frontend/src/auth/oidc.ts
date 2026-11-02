export function getToken(): string | null {
  return localStorage.getItem("access_token");
}

function setTokens(data: {
  access_token: string;
  id_token?: string;
  refresh_token?: string;
}) {
  localStorage.setItem("access_token", data.access_token);
  if (data.id_token) localStorage.setItem("id_token", data.id_token);
  if (data.refresh_token)
    localStorage.setItem("refresh_token", data.refresh_token);
}

export function clearTokens() {
  localStorage.clear();
  sessionStorage.clear();
}

export function authHeader(): Record<string, string> {
  const t = getToken() ?? localStorage.getItem("id_token");
  return t ? { Authorization: `Bearer ${t}` } : {};
}

export function installAuthFetch() {
  const orig = window.fetch;
  window.fetch = async (input: RequestInfo | URL, init: RequestInit = {}) => {
    const headers = new Headers(init.headers as HeadersInit);
    const ah = authHeader();
    for (const [k, v] of Object.entries(ah))
      if (!headers.has(k)) headers.set(k, v);
    const resp = await orig(input, { ...init, headers });
    if (
      resp.status === 401 &&
      !String(input).includes("/api/auth/config") &&
      !String(input).includes("/api/auth/callback") &&
      !String(input).includes("/api/auth/me") &&
      !String(input).includes("/api/oidc/login")
    ) {
      if (sessionStorage.getItem("login_in_progress") !== "1") {
        clearTokens();
        sessionStorage.setItem("login_in_progress", "1");
        login();
      }
    }
    return resp;
  };
}

async function fetchAuthConfig(): Promise<{
  issuer: string;
  client_id: string;
  redirect_uri: string;
}> {
  const r = await fetch("/api/auth/config");
  if (!r.ok) throw new Error("auth config failed");
  return r.json();
}

function b64url(bytes: Uint8Array): string {
  return btoa(String.fromCharCode(...bytes))
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "");
}

async function savePkce(): Promise<string> {
  const verifier = b64url(crypto.getRandomValues(new Uint8Array(32)));
  const digest = await crypto.subtle.digest(
    "SHA-256",
    new TextEncoder().encode(verifier),
  );
  sessionStorage.setItem("oidc_verifier", verifier);
  return b64url(new Uint8Array(digest));
}

export async function login() {
  const cfg = await fetchAuthConfig();
  const state =
    globalThis.crypto?.randomUUID?.() ??
    Math.random().toString(36).slice(2) + Math.random().toString(36).slice(2);
  const safeState = state.replace(/-/g, "").slice(0, 32).padEnd(16, "0");
  sessionStorage.setItem("oidc_state", safeState);
  const challenge = await savePkce();
  const url = new URL(`${cfg.issuer}/api/oidc/authorization`);
  url.searchParams.set("client_id", cfg.client_id);
  url.searchParams.set("redirect_uri", cfg.redirect_uri);
  url.searchParams.set("response_type", "code");
  url.searchParams.set("scope", "openid groups email profile");
  url.searchParams.set("state", safeState);
  url.searchParams.set("code_challenge", challenge);
  url.searchParams.set("code_challenge_method", "S256");
  window.location.href = url.toString();
}

export async function handleCallback(code: string): Promise<void> {
  const cfg = await fetchAuthConfig();
  const verifier = sessionStorage.getItem("oidc_verifier");
  const r = await fetch("/api/auth/callback", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      code,
      redirect_uri: cfg.redirect_uri,
      ...(verifier ? { code_verifier: verifier } : {}),
    }),
  });
  if (!r.ok) {
    const txt = await r.text();
    throw new Error(txt);
  }
  const data = await r.json();
  setTokens(data);
}

export async function logout() {
  const { OIDC_ISSUER_URL, APP_URL } = await import("../env");
  clearTokens();
  sessionStorage.setItem("logged_out", "1");
  window.location.href = `${OIDC_ISSUER_URL}/logout?rd=${encodeURIComponent(`${APP_URL}/`)}`;
}
