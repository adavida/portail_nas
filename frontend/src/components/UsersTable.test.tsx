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
