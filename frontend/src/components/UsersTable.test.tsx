import { render, screen } from "@testing-library/react";
import { expect, test } from "vitest";
import { UsersTable } from "./UsersTable";

test("empty", () => {
  render(<UsersTable users={[]} />);

  expect(screen.getByTestId("users-empty")).toBeInTheDocument();
});

test("renders rows", () => {
  render(
    <UsersTable
      users={[
        { uid: "alice", name: "Alice Dupont", email: "alice@example.com" },
        { uid: "bob", name: "Bob", email: "" },
      ]}
    />,
  );

  expect(screen.getByTestId("users-table")).toBeInTheDocument();
  expect(screen.getByTestId("user-row-alice")).toHaveTextContent("alice");
  expect(screen.getByTestId("user-row-bob")).toHaveTextContent("Bob");
});

test("create row is first line of table", () => {
  const { container } = render(<UsersTable users={[]} onCreated={() => {}} />);

  const rows = container.querySelectorAll("tbody tr");

  expect(rows[0].getAttribute("data-testid")).toBe("create-row");
  expect(screen.getByTestId("input-uid")).toBeInTheDocument();
  expect(screen.getByTestId("input-password")).toBeInTheDocument();
});

test("existing user has password field", () => {
  render(
    <UsersTable
      users={[{ uid: "alice", name: "Alice", email: "a@ex.com" }]}
      onCreated={() => {}}
    />,
  );

  expect(screen.getByTestId("password-input-alice")).toBeInTheDocument();
  expect(screen.getByTestId("password-button-alice")).toBeInTheDocument();
});
