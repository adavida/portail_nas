import { fireEvent, render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import { GroupsTable } from "./GroupsTable";

test("empty", () => {
  render(<GroupsTable groups={[]} />);

  expect(screen.getByTestId("groups-empty")).toBeInTheDocument();
});

test("renders rows with gid, name, description and delete", () => {
  const groups = [{ gid: "devs", name: "Devs", description: "Devs team" }];

  render(<GroupsTable groups={groups} />);

  expect(screen.getByTestId("groups-table")).toBeInTheDocument();
  expect(screen.getByTestId("group-row-devs")).toHaveTextContent("Devs");
  expect(screen.getByTestId("group-delete-button-devs")).toBeInTheDocument();
});

test("delete calls api and refreshes", async () => {
  const onDeleted = vi.fn();
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({ ok: true, json: () => Promise.resolve({}) } as Response),
  );

  const groups = [{ gid: "devs", name: "Devs", description: "Devs team" }];

  render(<GroupsTable groups={groups} onDeleted={onDeleted} />);

  fireEvent.click(screen.getByTestId("group-delete-button-devs"));

  await vi.waitFor(() => {
    expect(globalThis.fetch).toHaveBeenCalledWith(
      "/api/groups/devs",
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

  const groups = [{ gid: "devs", name: "Devs", description: "Devs team" }];

  render(<GroupsTable groups={groups} />);

  fireEvent.click(screen.getByTestId("group-delete-button-devs"));

  const error = await screen.findByTestId("group-error-devs");

  expect(error).toHaveTextContent("ldap: boom");
});

test("no create row: creation is done from the users page", () => {
  render(<GroupsTable groups={[]} onDeleted={() => {}} />);

  expect(screen.queryByTestId("group-create-row")).not.toBeInTheDocument();
  expect(screen.queryByTestId("input-member")).not.toBeInTheDocument();
});

test("edit name sends PUT with name and current description", async () => {
  const fetchMock = vi.fn(() =>
    Promise.resolve({ ok: true, json: () => Promise.resolve({}) } as Response),
  );
  globalThis.fetch = fetchMock;

  const groups = [{ gid: "devs", name: "Devs", description: "Devs team" }];

  render(<GroupsTable groups={groups} onUpdated={() => {}} />);

  fireEvent.doubleClick(screen.getByTestId("cell-name-devs"));
  fireEvent.change(screen.getByTestId("edit-input-name-devs"), {
    target: { value: "Renamed" },
  });
  fireEvent.keyDown(screen.getByTestId("edit-input-name-devs"), {
    key: "Enter",
  });

  await vi.waitFor(() => {
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
    Promise.resolve({ ok: true, json: () => Promise.resolve({}) } as Response),
  );
  globalThis.fetch = fetchMock;

  const groups = [{ gid: "devs", name: "Devs", description: "Devs team" }];

  render(<GroupsTable groups={groups} onUpdated={() => {}} />);

  fireEvent.doubleClick(screen.getByTestId("cell-description-devs"));
  fireEvent.change(screen.getByTestId("edit-input-description-devs"), {
    target: { value: "" },
  });
  fireEvent.keyDown(screen.getByTestId("edit-input-description-devs"), {
    key: "Enter",
  });

  await vi.waitFor(() => {
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/groups/devs",
      expect.objectContaining({
        method: "PUT",
        body: JSON.stringify({ name: "Devs", description: "" }),
      }),
    );
  });
});
