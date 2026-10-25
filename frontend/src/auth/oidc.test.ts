// @ts-expect-error — node:crypto missing from the DOM tsconfig (jsdom)
import { webcrypto as nodeCrypto } from "node:crypto";
import { beforeEach, expect, test } from "vitest";
import { b64url } from "../test/b64url";
import { mockFetch } from "../test/mock_fetch";
import { redirectTarget } from "../test/redirect_target";
import { handleCallback } from "./oidc";

beforeEach(() => {
  sessionStorage.clear();
  localStorage.clear();
  Object.defineProperty(globalThis, "crypto", {
    configurable: true,
    value: nodeCrypto,
  });
});

test("login adds PKCE S256 challenge and stores verifier", async () => {
  mockFetch([]);

  const url = await redirectTarget();

  const verifier = sessionStorage.getItem("oidc_verifier");
  expect(verifier, "verifier should be stored in sessionStorage").toBeTruthy();
  expect(url.searchParams.get("code_challenge_method")).toBe("S256");

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
