import { render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import { mockFetchByUrl } from "../test/mock_fetch_by_url";
import Users from "./Users";

test("loading then data", async () => {
  globalThis.fetch = mockFetchByUrl({
    "/api/users": [
      {
        uid: "alice",
        name: "Alice",
        email: "alice@example.com",
        groups: ["devs"],
      },
    ],
    "/api/groups": [
      {
        gid: "devs",
        name: "Devs",
        description: "",
      },
    ],
  });

  render(<Users />);

  expect(screen.getByTestId("users-loading")).toBeInTheDocument();

  const table = await screen.findByTestId("users-table");
  const row = screen.getByTestId("user-row-alice");

  expect(table).toBeInTheDocument();
  expect(row).toHaveTextContent("Alice");
  expect(row).toHaveTextContent("devs");
});

test("empty shows message", async () => {
  globalThis.fetch = mockFetchByUrl({});

  render(<Users />);

  const empty = await screen.findByTestId("users-empty");

  expect(empty).toHaveTextContent("Aucun utilisateur");
});

test("error", async () => {
  globalThis.fetch = vi.fn(() => Promise.reject(new Error("fail")));

  render(<Users />);

  const error = await screen.findByTestId("users-error");

  expect(error).toHaveTextContent("Erreur");
});
