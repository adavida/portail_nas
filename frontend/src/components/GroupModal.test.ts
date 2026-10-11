import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { expect, test, vi } from "vitest";
import { renderWithToaster } from "../test/render_with_toaster";
import GroupModal from "./GroupModal.svelte";

const devs = { gid: "devs", name: "Devs", description: "Devs team" };

test("valider and cancel are shown under name and under description", () => {
  render(GroupModal, {
    props: { group: devs, canDelete: true, onClose: () => {} },
  });

  expect(screen.getByTestId("modal-name-devs-save")).toBeInTheDocument();
  expect(screen.getByTestId("modal-name-devs-cancel")).toBeInTheDocument();
  expect(screen.getByTestId("modal-description-devs-save")).toBeInTheDocument();
  expect(
    screen.getByTestId("modal-description-devs-cancel"),
  ).toBeInTheDocument();
});

test("save sends PUT with name and description", async () => {
  const fetchMock = vi.fn(() =>
    Promise.resolve({
      ok: true,
      json: () => Promise.resolve({}),
    } as unknown as Response),
  );
  globalThis.fetch = fetchMock;
  const onUpdated = vi.fn();

  render(GroupModal, {
    props: { group: devs, canDelete: true, onClose: () => {}, onUpdated },
  });
  await fireEvent.input(screen.getByTestId("modal-name-devs"), {
    target: { value: "Renamed" },
  });
  await fireEvent.input(screen.getByTestId("modal-description-devs"), {
    target: { value: "Renamed team" },
  });
  await fireEvent.click(screen.getByTestId("modal-name-devs-save"));

  await waitFor(() => {
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/groups/devs",
      expect.objectContaining({
        method: "PUT",
        body: JSON.stringify({ name: "Renamed", description: "Renamed team" }),
      }),
    );
    expect(onUpdated).toHaveBeenCalled();
  });
});

test("save error keeps modal open with message", async () => {
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({
      ok: false,
      status: 500,
      json: () => Promise.resolve({ error: "name is empty" }),
    } as unknown as Response),
  );

  renderWithToaster(GroupModal, {
    group: devs,
    canDelete: true,
    onClose: () => {},
  });
  await fireEvent.input(screen.getByTestId("modal-name-devs"), {
    target: { value: "" },
  });
  await fireEvent.click(screen.getByTestId("modal-name-devs-save"));

  const error = await screen.findByTestId("toast-error");

  expect(error).toHaveTextContent("name is empty");
  expect(screen.getByTestId("group-modal-devs")).toBeInTheDocument();
});

test("delete calls api, refreshes and closes", async () => {
  const fetchMock = vi.fn(() =>
    Promise.resolve({
      ok: true,
      json: () => Promise.resolve({}),
    } as unknown as Response),
  );
  globalThis.fetch = fetchMock;
  const onDeleted = vi.fn();
  const onClose = vi.fn();

  render(GroupModal, {
    props: { group: devs, canDelete: true, onClose, onDeleted },
  });
  await fireEvent.click(screen.getByTestId("group-delete-button-devs"));
  await fireEvent.click(screen.getByTestId("confirm-yes"));

  await waitFor(() => {
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/groups/devs",
      expect.objectContaining({ method: "DELETE" }),
    );
    expect(onDeleted).toHaveBeenCalled();
    expect(onClose).toHaveBeenCalled();
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
  const onClose = vi.fn();

  renderWithToaster(GroupModal, {
    group: devs,
    canDelete: true,
    onClose,
  });
  await fireEvent.click(screen.getByTestId("group-delete-button-devs"));
  await fireEvent.click(screen.getByTestId("confirm-yes"));

  const error = await screen.findByTestId("toast-error");

  expect(error).toHaveTextContent("ldap: boom");
  expect(onClose).not.toHaveBeenCalled();
});

test("protected group has no delete button", () => {
  const admin = { gid: "admin", name: "Admin", description: "" };

  render(GroupModal, {
    props: { group: admin, canDelete: false, onClose: () => {} },
  });

  expect(
    screen.queryByTestId("group-delete-button-admin"),
  ).not.toBeInTheDocument();
});
