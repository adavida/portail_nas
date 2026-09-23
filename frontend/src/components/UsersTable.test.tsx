import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import { UsersTable } from "./UsersTable";
import { Toaster } from "./Toaster";

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
  expect(screen.queryByTestId("input-user-group")).not.toBeInTheDocument();
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

  await waitFor(() => {
    expect(globalThis.fetch).toHaveBeenCalledWith(
      "/api/users/alice",
      expect.objectContaining({ method: "DELETE" }),
    );
  });

  await waitFor(() => {
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

  render(
    <>
      <UsersTable users={[alice]} />
      <Toaster />
    </>,
  );

  fireEvent.click(screen.getByTestId("delete-button-alice"));

  const error = await screen.findByTestId("toast-error");

  expect(error).toHaveTextContent("ldap: boom");
});

test("double-click name opens editor then saves", async () => {
  const onUpdated = vi.fn();
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({ ok: true, json: () => Promise.resolve({}) } as Response),
  );

  const alice = { uid: "alice", name: "Alice", email: "a@ex.com" };

  render(<UsersTable users={[alice]} onUpdated={onUpdated} />);

  fireEvent.doubleClick(screen.getByTestId("cell-name-alice"));

  const input = screen.getByTestId("edit-input-name-alice");

  fireEvent.change(input, { target: { value: "Alice Dupont" } });
  fireEvent.keyDown(input, { key: "Enter" });

  await waitFor(() => {
    expect(globalThis.fetch).toHaveBeenCalledWith(
      "/api/users/alice",
      expect.objectContaining({
        method: "PUT",
        body: JSON.stringify({ name: "Alice Dupont", email: "a@ex.com" }),
      }),
    );
  });

  await waitFor(() => {
    expect(onUpdated).toHaveBeenCalled();
  });
});

test("edit error keeps editor open", async () => {
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({
      ok: false,
      status: 500,
      json: () => Promise.resolve({ error: "name is empty" }),
    } as Response),
  );

  const alice = { uid: "alice", name: "Alice", email: "a@ex.com" };

  render(
    <>
      <UsersTable users={[alice]} />
      <Toaster />
    </>,
  );

  fireEvent.doubleClick(screen.getByTestId("cell-name-alice"));
  fireEvent.change(screen.getByTestId("edit-input-name-alice"), {
    target: { value: "" },
  });
  fireEvent.keyDown(screen.getByTestId("edit-input-name-alice"), {
    key: "Enter",
  });

  const error = await screen.findByTestId("toast-error");

  expect(error).toHaveTextContent("name is empty");
  expect(screen.getByTestId("edit-input-name-alice")).toBeInTheDocument();
});

test("groups cell shows memberships", () => {
  const alice = { uid: "alice", name: "Alice", email: "", groups: ["devs"] };

  render(
    <>
      <UsersTable users={[alice]} />
      <Toaster />
    </>,
  );

  expect(screen.getByTestId("cell-groups-alice")).toHaveTextContent("devs");
});

test("groups cell edit proposes groups and diffs membership", async () => {
  const fetchMock = vi.fn(() =>
    Promise.resolve({ ok: true, json: () => Promise.resolve({}) } as Response),
  );
  globalThis.fetch = fetchMock;
  const onUpdated = vi.fn();

  const alice = { uid: "alice", name: "Alice", email: "", groups: ["devs"] };

  render(
    <UsersTable
      users={[alice]}
      allGroups={["devs", "ops"]}
      onUpdated={onUpdated}
    />,
  );

  fireEvent.doubleClick(screen.getByTestId("cell-groups-alice"));

  expect(
    Array.from(
      screen
        .getByTestId("groups-editor-alice")
        .querySelectorAll("[data-testid^='groups-chip-']"),
    ).map((el) => el.getAttribute("data-testid")),
  ).toContain("groups-chip-alice-devs");

  fireEvent.change(screen.getByTestId("groups-new-input-alice"), {
    target: { value: "ops" },
  });
  fireEvent.click(screen.getByTestId("groups-suggest-alice-ops"));
  fireEvent.click(screen.getByTestId("groups-remove-alice-devs"));
  fireEvent.click(screen.getByTestId("groups-save-alice"));

  await waitFor(() => {
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/groups/ops/members",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ uid: "alice" }),
      }),
    );

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/groups/devs/members/alice",
      expect.objectContaining({ method: "DELETE" }),
    );
  });

  await waitFor(() => {
    expect(onUpdated).toHaveBeenCalled();
  });
});

test("groups save error keeps editor open with message", async () => {
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({
      ok: false,
      status: 404,
      json: () => Promise.resolve({ error: "not found: uid=ghost" }),
    } as Response),
  );

  const alice = { uid: "alice", name: "Alice", email: "", groups: [] };

  render(
    <>
      <UsersTable users={[alice]} allGroups={["devs"]} />
      <Toaster />
    </>,
  );

  fireEvent.doubleClick(screen.getByTestId("cell-groups-alice"));
  fireEvent.change(screen.getByTestId("groups-new-input-alice"), {
    target: { value: "devs" },
  });
  fireEvent.keyDown(screen.getByTestId("groups-new-input-alice"), {
    key: "Enter",
  });
  fireEvent.click(screen.getByTestId("groups-save-alice"));

  const error = await screen.findByTestId("toast-error");

  expect(error).toHaveTextContent("not found: uid=ghost");
  expect(screen.getByTestId("groups-save-alice")).toBeInTheDocument();
});

test("create group from groups cell with user as member", async () => {
  const fetchMock = vi.fn(() =>
    Promise.resolve({ ok: true, json: () => Promise.resolve({}) } as Response),
  );
  globalThis.fetch = fetchMock;

  const alice = { uid: "alice", name: "Alice", email: "", groups: [] };

  render(<UsersTable users={[alice]} onUpdated={() => {}} />);

  fireEvent.doubleClick(screen.getByTestId("cell-groups-alice"));
  fireEvent.change(screen.getByTestId("groups-new-input-alice"), {
    target: { value: "devs" },
  });
  fireEvent.keyDown(screen.getByTestId("groups-new-input-alice"), {
    key: "Enter",
  });

  await waitFor(() => {
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/groups",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({
          gid: "devs",
          name: "devs",
          description: "",
          members: ["alice"],
        }),
      }),
    );
  });
});

test("create group error keeps input and shows message", async () => {
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({
      ok: false,
      status: 500,
      json: () => Promise.resolve({ error: "ldap: boom" }),
    } as Response),
  );

  const alice = { uid: "alice", name: "Alice", email: "", groups: [] };

  render(
    <>
      <UsersTable users={[alice]} />
      <Toaster />
    </>,
  );

  fireEvent.doubleClick(screen.getByTestId("cell-groups-alice"));
  fireEvent.change(screen.getByTestId("groups-new-input-alice"), {
    target: { value: "devs" },
  });
  fireEvent.keyDown(screen.getByTestId("groups-new-input-alice"), {
    key: "Enter",
  });

  const error = await screen.findByTestId("toast-error");

  expect(error).toHaveTextContent("ldap: boom");
  expect(screen.getByTestId("groups-new-input-alice")).toBeInTheDocument();
});
