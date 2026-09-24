// PKCE: login() doit générer + stocker un code_verifier et poser
// code_challenge/code_challenge_method sur l'URL d'autorisation Authelia;
// handleCallback() doit renvoyer le verifier au backend.
// @ts-expect-error — node:crypto absent du tsconfig DOM (jsdom)
import { webcrypto as nodeCrypto } from "node:crypto";
import { beforeEach, expect, test, vi } from "vitest";
import { handleCallback, login } from "./oidc";

function b64url(bytes: Uint8Array): string {
  return btoa(String.fromCharCode(...bytes))
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "");
}

// jsdom ne fournit pas WebCrypto — on injecte node:crypto/webcrypto.
beforeEach(() => {
  sessionStorage.clear();
  localStorage.clear();
  Object.defineProperty(globalThis, "crypto", {
    configurable: true,
    value: nodeCrypto,
  });
});

function mockFetch(calls: Array<{ url: string; body?: unknown }>) {
  globalThis.fetch = vi.fn(((url: RequestInfo | URL, init?: RequestInit) => {
    const url_ = String(url);
    calls.push({ url: url_, body: init?.body });
    if (url_.includes("/api/auth/config"))
      return Promise.resolve({
        ok: true,
        json: async () => ({
          issuer: "https://127.0.0.1:9091",
          client_id: "portail-dev",
          redirect_uri: "http://localhost:5173/callback",
        }),
      } as unknown as Response);
    return Promise.resolve({
      ok: true,
      json: async () => ({ access_token: "at" }),
    } as unknown as Response);
  }) as typeof fetch);
}

async function redirectTarget(): Promise<URL> {
  let href = "";
  Object.defineProperty(window, "location", {
    configurable: true,
    value: {
      get href() {
        return href;
      },
      set href(v: string) {
        href = v;
      },
    },
  });
  await login();
  return new URL(href);
}

test("login adds PKCE S256 challenge and stores verifier", async () => {
  mockFetch([]);

  const url = await redirectTarget();

  const verifier = sessionStorage.getItem("oidc_verifier");
  expect(verifier, "verifier should be stored in sessionStorage").toBeTruthy();
  expect(url.searchParams.get("code_challenge_method")).toBe("S256");

  // Arrange: expected challenge = base64url(SHA-256(verifier))
  const digest = await nodeCrypto.subtle.digest(
    "SHA-256",
    new TextEncoder().encode(verifier!),
  );
  const expected = b64url(new Uint8Array(digest));

  expect(url.searchParams.get("code_challenge")).toBe(expected);
});

test("handleCallback sends the stored verifier to the backend", async () => {
  const calls: Array<{ url: string; body?: unknown }> = [];
  mockFetch(calls);

  sessionStorage.setItem("oidc_verifier", "my-verifier-123");
  await handleCallback("code-abc");

  const body = JSON.parse(String(calls[1]?.body)) as Record<string, string>;
  expect(body.code_verifier, "verifier should be forwarded").toBe(
    "my-verifier-123",
  );
});
