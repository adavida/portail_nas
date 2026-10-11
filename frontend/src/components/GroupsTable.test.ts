import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { expect, test, vi } from "vitest";
import { renderWithToaster } from "../test/render_with_toaster";
import GroupsTable from "./GroupsTable.svelte";

test("empty", () => {
  render(GroupsTable, { props: { groups: [] } });

  expect(screen.getByTestId("groups-empty")).toBeInTheDocument();
});

test("renders rows with gid, name, description and delete", () => {
  const groups = [{ gid: "devs", name: "Devs", description: "Devs team" }];

  render(GroupsTable, { props: { groups } });

  expect(screen.getByTestId("groups-table")).toBeInTheDocument();
  expect(screen.getByTestId("group-row-devs")).toHaveTextContent("Devs");
  expect(screen.getByTestId("group-delete-button-devs")).toBeInTheDocument();
});

test("user row is hidden, admin row has no delete button", () => {
  const groups = [
    { gid: "admin", name: "Admin", description: "" },
    { gid: "user", name: "User", description: "" },
    { gid: "devs", name: "Devs", description: "" },
  ];

  render(GroupsTable, { props: { groups } });

  expect(screen.queryByTestId("group-row-user")).toBeNull();
  expect(screen.getByTestId("group-row-admin")).toBeInTheDocument();
  expect(screen.queryByTestId("group-delete-button-admin")).toBeNull();
  expect(screen.getByTestId("group-delete-button-devs")).toBeInTheDocument();
});

test("delete calls api and refreshes", async () => {
  const onDeleted = vi.fn();
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({
      ok: true,
      json: () => Promise.resolve({}),
    } as unknown as Response),
  );

  const groups = [{ gid: "devs", name: "Devs", description: "Devs team" }];

  render(GroupsTable, { props: { groups, onDeleted } });

  await fireEvent.click(screen.getByTestId("group-delete-button-devs"));

  await waitFor(() => {
    expect(globalThis.fetch).toHaveBeenCalledWith(
      "/api/groups/devs",
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
    } as unknown as Response),
  );

  const groups = [{ gid: "devs", name: "Devs", description: "Devs team" }];

  renderWithToaster(GroupsTable, { groups });

  await fireEvent.click(screen.getByTestId("group-delete-button-devs"));

  const error = await screen.findByTestId("toast-error");

  expect(error).toHaveTextContent("ldap: boom");
});

test("no create row: creation is done from the users page", () => {
  render(GroupsTable, { props: { groups: [], onDeleted: () => {} } });

  expect(screen.queryByTestId("group-create-row")).not.toBeInTheDocument();
  expect(screen.queryByTestId("input-member")).not.toBeInTheDocument();
});

test("edit name sends PUT with name and current description", async () => {
  const fetchMock = vi.fn(() =>
    Promise.resolve({
      ok: true,
      json: () => Promise.resolve({}),
    } as unknown as Response),
  );
  globalThis.fetch = fetchMock;

  const groups = [{ gid: "devs", name: "Devs", description: "Devs team" }];

  render(GroupsTable, { props: { groups, onUpdated: () => {} } });

  await fireEvent.click(screen.getByTestId("cell-name-devs"));
  const input = screen.getByTestId("edit-input-name-devs");
  await fireEvent.input(input, { target: { value: "Renamed" } });
  await fireEvent.keyDown(input, { key: "Enter" });

  await waitFor(() => {
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/groups/devs",
      expect.objectContaining({
        method: "PUT",
        body: JSON.stringify({ name: "Renamed", description: "Devs team" }),
      }),
    );
  });
});

test("edit description sends PUT with description and current name", async () => {
  const fetchMock = vi.fn(() =>
    Promise.resolve({
      ok: true,
      json: () => Promise.resolve({}),
    } as unknown as Response),
  );
  globalThis.fetch = fetchMock;

  const groups = [{ gid: "devs", name: "Devs", description: "Devs team" }];

  render(GroupsTable, { props: { groups, onUpdated: () => {} } });

  await fireEvent.click(screen.getByTestId("cell-description-devs"));
  const input = screen.getByTestId("edit-input-description-devs");
  await fireEvent.input(input, { target: { value: "" } });
  await fireEvent.keyDown(input, { key: "Enter" });

  await waitFor(() => {
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/groups/devs",
      expect.objectContaining({
        method: "PUT",
        body: JSON.stringify({ name: "Devs", description: "" }),
      }),
    );
  });
});
