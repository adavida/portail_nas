import { fireEvent, render, screen } from "@testing-library/svelte";
import { expect, test, vi } from "vitest";
import { mockFetchByUrl } from "../test/mock_fetch_by_url";
import DebugClaims from "./DebugClaims.svelte";

const users = [{ uid: "alice", name: "Alice", email: "" }];
const clients = [
  {
    id: "portail-dev",
    enabled: true,
    has_secret: true,
    redirect_uris: [],
    scopes: ["openid"],
    required_group: "",
    forbidden_group: "",
  },
];

test("selecting user and client shows the claims", async () => {
  globalThis.fetch = mockFetchByUrl({
    "/api/auth/debug/claims?uid=alice&client_id=portail-dev": {
      sub: "abc-123",
    },
  });

  render(DebugClaims, { users, clients });

  await fireEvent.change(screen.getByTestId("claims-user"), {
    target: { value: "alice" },
  });
  await fireEvent.change(screen.getByTestId("claims-client"), {
    target: { value: "portail-dev" },
  });

  const claims = await screen.findByTestId("debug-claims");

  expect(claims).toHaveTextContent("abc-123");
});

test("renders nothing until a user and a client are selected", () => {
  globalThis.fetch = vi.fn(() => Promise.reject(new Error("must not fetch")));

  render(DebugClaims, { users, clients });

  expect(screen.queryByTestId("claims-loading")).not.toBeInTheDocument();
  expect(screen.queryByTestId("debug-claims")).not.toBeInTheDocument();
});
