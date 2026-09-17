import { render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import Users from "./Users";

test("loading then data", async () => {
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({
      ok: true,
      json: () =>
        Promise.resolve([
          { uid: "alice", name: "Alice", email: "alice@example.com" },
        ]),
    } as Response),
  );
  render(<Users />);
  expect(screen.getByTestId("users-loading")).toBeInTheDocument();
  expect(await screen.findByTestId("users-table")).toBeInTheDocument();
  expect(screen.getByTestId("user-row-alice")).toBeInTheDocument();
});

test("empty shows message", async () => {
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({ ok: true, json: () => Promise.resolve([]) } as Response),
  );
  render(<Users />);
  expect(await screen.findByTestId("users-empty")).toBeInTheDocument();
});

test("error", async () => {
  globalThis.fetch = vi.fn(() => Promise.reject(new Error("fail")));
  render(<Users />);
  expect(await screen.findByTestId("users-error")).toBeInTheDocument();
});
