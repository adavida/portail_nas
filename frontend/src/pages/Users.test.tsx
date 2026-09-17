import { render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import Users from "./Users";

test("loading then data", async () => {
  const mockUsers = [
    { uid: "alice", name: "Alice", email: "alice@example.com" },
  ];
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({
      ok: true,
      json: () => Promise.resolve(mockUsers),
    } as Response),
  );

  render(<Users />);

  expect(screen.getByTestId("users-loading")).toBeInTheDocument();

  const table = await screen.findByTestId("users-table");
  const row = screen.getByTestId("user-row-alice");

  expect(table).toBeInTheDocument();
  expect(row).toHaveTextContent("Alice");
});

test("empty shows message", async () => {
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({ ok: true, json: () => Promise.resolve([]) } as Response),
  );

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
