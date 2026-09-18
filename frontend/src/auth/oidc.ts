export function getToken(): string | null {
  return localStorage.getItem("access_token");
}

export function setTokens(data: {
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
  localStorage.removeItem("access_token");
  localStorage.removeItem("id_token");
  localStorage.removeItem("refresh_token");
}

export function authHeader(): Record<string, string> {
  const t = localStorage.getItem("id_token") ?? getToken();
  return t ? { Authorization: `Bearer ${t}` } : {};
}

export function installAuthFetch() {
  const orig = window.fetch;
  window.fetch = (input: RequestInfo | URL, init: RequestInit = {}) => {
    const headers = new Headers(init.headers as HeadersInit);
    const ah = authHeader();
    for (const [k, v] of Object.entries(ah))
      if (!headers.has(k)) headers.set(k, v);
    return orig(input, { ...init, headers });
  };
}

export async function fetchAuthConfig(): Promise<{
  issuer: string;
  client_id: string;
  redirect_uri: string;
}> {
  const r = await fetch("/api/auth/config");
  if (!r.ok) throw new Error("auth config failed");
  return r.json();
}

export async function login() {
  const cfg = await fetchAuthConfig();
  const state =
    globalThis.crypto?.randomUUID?.() ??
    Math.random().toString(36).slice(2) + Math.random().toString(36).slice(2);
  // Authelia exige state >= 8 chars
  const safeState = state.replace(/-/g, "").slice(0, 32).padEnd(16, "0");
  sessionStorage.setItem("oidc_state", safeState);
  const url = new URL(`${cfg.issuer}/api/oidc/authorization`);
  url.searchParams.set("client_id", cfg.client_id);
  url.searchParams.set("redirect_uri", cfg.redirect_uri);
  url.searchParams.set("response_type", "code");
  url.searchParams.set("scope", "openid groups email profile offline_access");
  url.searchParams.set("state", safeState);
  window.location.href = url.toString();
}

export async function handleCallback(code: string): Promise<void> {
  const cfg = await fetchAuthConfig();
  const r = await fetch("/api/auth/callback", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ code, redirect_uri: cfg.redirect_uri }),
  });
  if (!r.ok) {
    const txt = await r.text();
    throw new Error(txt);
  }
  const data = await r.json();
  setTokens(data);
}
