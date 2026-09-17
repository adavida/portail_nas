import { fireEvent, render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import { UsersTable } from "./UsersTable";

test("empty", () => {
  render(<UsersTable users={[]} />);

  expect(screen.getByTestId("users-empty")).toBeInTheDocument();
});

test("renders rows", () => {
  const users = [
    { uid: "alice", name: "Alice Dupont", email: "alice@example.com" },
    { uid: "bob", name: "Bob", email: "" },
  ];

  render(<UsersTable users={users} />);

  expect(screen.getByTestId("users-table")).toBeInTheDocument();
  expect(screen.getByTestId("user-row-alice")).toHaveTextContent("alice");
  expect(screen.getByTestId("user-row-bob")).toHaveTextContent("Bob");
});

test("create row is first line of table", () => {
  const { container } = render(<UsersTable users={[]} onCreated={() => {}} />);

  const rows = container.querySelectorAll("tbody tr");
  const firstRowId = rows[0].getAttribute("data-testid");

  expect(firstRowId).toBe("create-row");
  expect(screen.getByTestId("input-uid")).toBeInTheDocument();
  expect(screen.getByTestId("input-password")).toBeInTheDocument();
});

test("existing user has password field", () => {
  const alice = { uid: "alice", name: "Alice", email: "a@ex.com" };

  render(<UsersTable users={[alice]} onCreated={() => {}} />);

  expect(screen.getByTestId("password-input-alice")).toBeInTheDocument();
  expect(screen.getByTestId("password-button-alice")).toBeInTheDocument();
});

test("delete user calls api and refreshes", async () => {
  const onDeleted = vi.fn();
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({ ok: true, json: () => Promise.resolve({}) } as Response),
  );

  const alice = { uid: "alice", name: "Alice", email: "a@ex.com" };

  render(<UsersTable users={[alice]} onDeleted={onDeleted} />);

  fireEvent.click(screen.getByTestId("delete-button-alice"));

  await vi.waitFor(() => {
    expect(globalThis.fetch).toHaveBeenCalledWith(
      "/api/users/alice",
      expect.objectContaining({ method: "DELETE" }),
    );
  });

  await vi.waitFor(() => {
    expect(onDeleted).toHaveBeenCalled();
  });
});

test("delete shows error on failure", async () => {
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({
      ok: false,
      status: 500,
      json: () => Promise.resolve({ error: "ldap: boom" }),
    } as Response),
  );

  const alice = { uid: "alice", name: "Alice", email: "a@ex.com" };

  render(<UsersTable users={[alice]} />);

  fireEvent.click(screen.getByTestId("delete-button-alice"));

  const error = await screen.findByTestId("password-error-alice");

  expect(error).toHaveTextContent("ldap: boom");
});
