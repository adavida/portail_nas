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
