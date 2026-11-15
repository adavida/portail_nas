import { vi } from "vitest";

export function mockFetchByUrl(byUrl: Record<string, unknown>) {
  return vi.fn((url: RequestInfo | URL, init?: RequestInit) => {
    const method = (init?.method ?? "GET").toUpperCase();
    const body = byUrl[`${method} ${String(url)}`] ?? byUrl[String(url)] ?? [];
    return Promise.resolve({
      ok: true,
      json: () => Promise.resolve(body),
    } as unknown as Response);
  });
}
