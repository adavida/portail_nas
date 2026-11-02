import { fireEvent, render, screen } from "@testing-library/svelte";
import { expect, test, vi } from "vitest";
import { mockFetchByUrl } from "../test/mock_fetch_by_url";
import Debug from "./Debug.svelte";

const debugInfo = {
  issuer: "http://127.0.0.1:3000",
  client_id: "portail-dev",
  redirect_uri: "http://localhost:5173/callback",
  key_bits: 2048,
  pending_ttl_secs: 300,
  code_ttl_secs: 60,
  session_ttl_secs: 43200,
  access_ttl_secs: 3600,
  clients: [
    {
      id: "portail-dev",
      enabled: true,
      has_secret: true,
      redirect_uris: ["http://localhost:5173/callback"],
      scopes: ["openid", "groups"],
    },
  ],
};

test("loading then oidc info and apps", async () => {
  globalThis.fetch = mockFetchByUrl({
    "/api/auth/debug": debugInfo,
    "/apps.json": [{ name: "Nextcloud", url: "https://cloud.example/" }],
  });

  render(Debug);

  expect(screen.getByTestId("debug-loading")).toBeInTheDocument();

  const oidc = await screen.findByTestId("debug-oidc");
  const clientRow = screen.getByTestId("debug-client-0");
  const appItem = screen.getByTestId("debug-app-0");

  expect(oidc).toHaveTextContent("http://127.0.0.1:3000");
  expect(clientRow).toHaveTextContent("portail-dev");
  expect(appItem).toHaveTextContent("Nextcloud");
});

test("selecting user and client shows claims", async () => {
  globalThis.fetch = mockFetchByUrl({
    "/api/auth/debug": debugInfo,
    "/apps.json": [],
    "/api/users": [{ uid: "alice", name: "Alice", email: "" }],
    "/api/auth/debug/claims?uid=alice&client_id=portail-dev": {
      sub: "0f9e8d7c-6b5a-4938-2716-05f4e3d2c1b0",
      aud: "portail-dev",
      iss: "http://127.0.0.1:3000",
      preferred_username: "alice",
      groups: ["devs"],
    },
  });

  render(Debug);

  await screen.findByTestId("debug-oidc");
  await fireEvent.change(screen.getByTestId("claims-user"), {
    target: { value: "alice" },
  });
  await fireEvent.change(screen.getByTestId("claims-client"), {
    target: { value: "portail-dev" },
  });

  const claims = await screen.findByTestId("debug-claims");

  expect(claims).toHaveTextContent("0f9e8d7c-6b5a-4938-2716-05f4e3d2c1b0");
  expect(claims).toHaveTextContent("portail-dev");
});

test("error on fetch failure", async () => {
  globalThis.fetch = vi.fn(() => Promise.reject(new Error("fail")));

  render(Debug);

  const error = await screen.findByTestId("debug-error");

  expect(error).toHaveTextContent("Error");
});
