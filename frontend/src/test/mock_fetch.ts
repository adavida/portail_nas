import { vi } from "vitest";

export function mockFetch(calls: Array<{ url: string; body?: unknown }>) {
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
