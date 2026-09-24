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
  localStorage.clear();
  sessionStorage.clear();
}

export function authHeader(): Record<string, string> {
  const t = getToken() ?? localStorage.getItem("id_token");
  return t ? { Authorization: `Bearer ${t}` } : {};
}

export function isAdmin(): boolean {
  const token = localStorage.getItem("id_token") ?? getToken();
  if (!token) return false;
  try {
    const payload = token.split(".")[1];
    const json = JSON.parse(
      atob(payload.replace(/-/g, "+").replace(/_/g, "/")),
    );
    const groups: string[] = json.groups ?? [];
    return groups.includes("admin");
  } catch {
    return false;
  }
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
      !String(input).includes("/api/auth/me")
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

export async function fetchAuthConfig(): Promise<{
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

// PKCE S256: Authelia exige un code_challenge pour l'authorization_code flow.
// Sauve le verifier en sessionStorage — `login()` le crée, `handleCallback()`
// l'envoie au backend qui le passe à l'endpoint token.
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
  // Authelia exige state >= 8 chars
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
  // A: stay on the portal. Clear the storage immediately (instant feedback)
  // then silently destroy the Authelia session via iframe POST
  // (GET /api/logout = 405, fetch blocked by CORS).
  clearTokens();
  sessionStorage.setItem("logged_out", "1");
  try {
    const iframe = document.createElement("iframe");
    iframe.name = "authelia-logout-iframe";
    iframe.style.display = "none";
    document.body.appendChild(iframe);
    const form = document.createElement("form");
    form.method = "POST";
    form.action = `${OIDC_ISSUER_URL}/api/logout`;
    form.target = iframe.name;
    form.style.display = "none";
    document.body.appendChild(form);
    form.submit();
    await new Promise((r) => setTimeout(r, 500));
    iframe.remove();
    form.remove();
  } catch {
    try {
      await fetch(`${OIDC_ISSUER_URL}/api/logout`, {
        method: "POST",
        credentials: "include",
        mode: "no-cors",
      });
    } catch {
      // ignore
    }
  }
  window.location.href = APP_URL + "/";
}
