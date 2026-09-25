import { vi } from "vitest";

export function mockFetchByUrl(byUrl: Record<string, unknown>) {
  return vi.fn((_url: RequestInfo | URL) =>
    Promise.resolve({
      ok: true,
      json: () => Promise.resolve(byUrl[String(_url)] ?? []),
    } as unknown as Response),
  );
}
