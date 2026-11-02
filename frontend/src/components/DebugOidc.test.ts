import { render, screen } from "@testing-library/svelte";
import { expect, test } from "vitest";
import DebugOidc from "./DebugOidc.svelte";

const info = {
  issuer: "http://127.0.0.1:3000",
  client_id: "portail-dev",
  redirect_uri: "http://localhost:5173/callback",
  key_bits: 2048,
  pending_ttl_secs: 300,
  code_ttl_secs: 60,
  session_ttl_secs: 43200,
  access_ttl_secs: 3600,
  clients: [],
};

test("renders issuer and ttl metadata", () => {
  render(DebugOidc, { info });

  const dl = screen.getByTestId("debug-oidc");

  expect(dl).toHaveTextContent("http://127.0.0.1:3000");
  expect(dl).toHaveTextContent("2048");
  expect(dl).toHaveTextContent("43200");
});
