import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { expect, test, vi } from "vitest";
import { mockFetchByUrl } from "../test/mock_fetch_by_url";
import { renderWithToaster } from "../test/render_with_toaster";
import UserModal from "./UserModal.svelte";

const alice = { uid: "alice", name: "Alice", email: "a@ex.com" };

test("valider and cancel are shown under name and under email", () => {
  render(UserModal, { props: { user: alice, onClose: () => {} } });

  expect(screen.getByTestId("modal-name-alice-save")).toBeInTheDocument();
  expect(screen.getByTestId("modal-name-alice-cancel")).toBeInTheDocument();
  expect(screen.getByTestId("modal-email-alice-save")).toBeInTheDocument();
  expect(screen.getByTestId("modal-email-alice-cancel")).toBeInTheDocument();
});

test("save name sends PUT with name and current email", async () => {
  const fetchMock = vi.fn(() =>
    Promise.resolve({
      ok: true,
      json: () => Promise.resolve({}),
    } as unknown as Response),
  );
  globalThis.fetch = fetchMock;
  const onUpdated = vi.fn();

  render(UserModal, { props: { user: alice, onClose: () => {}, onUpdated } });
  await fireEvent.input(screen.getByTestId("modal-name-alice"), {
    target: { value: "Alice Dupont" },
  });
  await fireEvent.click(screen.getByTestId("modal-name-alice-save"));

  await waitFor(() => {
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/users/alice",
      expect.objectContaining({
        method: "PUT",
        body: JSON.stringify({ name: "Alice Dupont", email: "a@ex.com" }),
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

  renderWithToaster(UserModal, { user: alice, onClose: () => {} });
  await fireEvent.input(screen.getByTestId("modal-name-alice"), {
    target: { value: "" },
  });
  await fireEvent.click(screen.getByTestId("modal-name-alice-save"));

  const error = await screen.findByTestId("toast-error");

  expect(error).toHaveTextContent("name is empty");
  expect(screen.getByTestId("user-modal-alice")).toBeInTheDocument();
});

test("password update calls api", async () => {
  const fetchMock = vi.fn(() =>
    Promise.resolve({
      ok: true,
      json: () => Promise.resolve({}),
    } as unknown as Response),
  );
  globalThis.fetch = fetchMock;

  render(UserModal, { props: { user: alice, onClose: () => {} } });
  await fireEvent.input(screen.getByTestId("password-input-alice"), {
    target: { value: "s3cret" },
  });
  await fireEvent.click(screen.getByTestId("password-button-alice"));

  await waitFor(() => {
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/users/alice/password",
      expect.objectContaining({
        method: "PUT",
        body: JSON.stringify({ password: "s3cret" }),
      }),
    );
  });
});

test("empty password asks for a password", async () => {
  globalThis.fetch = vi.fn();

  renderWithToaster(UserModal, { user: alice, onClose: () => {} });
  await fireEvent.click(screen.getByTestId("password-button-alice"));

  const error = await screen.findByTestId("toast-error");

  expect(error).toHaveTextContent("password required");
  expect(globalThis.fetch).not.toHaveBeenCalled();
});

test("delete user calls api, refreshes and closes", async () => {
  const fetchMock = vi.fn(() =>
    Promise.resolve({
      ok: true,
      json: () => Promise.resolve({}),
    } as unknown as Response),
  );
  globalThis.fetch = fetchMock;
  const onDeleted = vi.fn();
  const onClose = vi.fn();

  render(UserModal, { props: { user: alice, onClose, onDeleted } });
  await fireEvent.click(screen.getByTestId("delete-button-alice"));

  await fireEvent.click(screen.getByTestId("confirm-yes"));

  await waitFor(() => {
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/users/alice",
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

  renderWithToaster(UserModal, { user: alice, onClose });
  await fireEvent.click(screen.getByTestId("delete-button-alice"));

  await fireEvent.click(screen.getByTestId("confirm-yes"));

  const error = await screen.findByTestId("toast-error");

  expect(error).toHaveTextContent("ldap: boom");
  expect(onClose).not.toHaveBeenCalled();
});

test("cancel of delete confirmation calls nothing", async () => {
  const fetchMock = vi.fn();
  globalThis.fetch = fetchMock;
  const onDeleted = vi.fn();

  render(UserModal, {
    props: { user: alice, onClose: () => {}, onDeleted },
  });
  await fireEvent.click(screen.getByTestId("delete-button-alice"));
  await fireEvent.click(screen.getByTestId("confirm-no"));

  expect(fetchMock).not.toHaveBeenCalled();
  expect(onDeleted).not.toHaveBeenCalled();
  expect(screen.getByTestId("user-modal-alice")).toBeInTheDocument();
});

test("annuler closes the modal", async () => {
  const onClose = vi.fn();

  render(UserModal, { props: { user: alice, onClose } });
  await fireEvent.click(screen.getByTestId("modal-name-alice-cancel"));

  expect(onClose).toHaveBeenCalled();
});

test("groups editor proposes groups and diffs membership", async () => {
  globalThis.fetch = mockFetchByUrl({});
  const fetchMock = globalThis.fetch as ReturnType<typeof vi.fn>;
  const onUpdated = vi.fn();
  const user = { ...alice, groups: ["devs"] };

  render(UserModal, {
    props: {
      user,
      allGroups: ["devs", "ops"],
      onClose: () => {},
      onUpdated,
    },
  });

  expect(screen.getByTestId("groups-editor-alice")).toBeInTheDocument();
  await fireEvent.input(screen.getByTestId("groups-new-input-alice"), {
    target: { value: "ops" },
  });
  await fireEvent.click(screen.getByTestId("groups-suggest-alice-ops"));
  await fireEvent.click(screen.getByTestId("groups-remove-alice-devs"));
  await fireEvent.click(screen.getByTestId("groups-save-alice"));

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
    expect(onUpdated).toHaveBeenCalled();
  });
});

test("groups save error keeps editor open with message", async () => {
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({
      ok: false,
      status: 404,
      json: () => Promise.resolve({ error: "not found: uid=ghost" }),
    } as unknown as Response),
  );
  const user = { ...alice, groups: [] };

  renderWithToaster(UserModal, {
    user,
    allGroups: ["devs"],
    onClose: () => {},
  });
  await fireEvent.input(screen.getByTestId("groups-new-input-alice"), {
    target: { value: "devs" },
  });
  await fireEvent.keyDown(screen.getByTestId("groups-new-input-alice"), {
    key: "Enter",
  });
  await fireEvent.click(screen.getByTestId("groups-save-alice"));

  const error = await screen.findByTestId("toast-error");

  expect(error).toHaveTextContent("not found: uid=ghost");
  expect(screen.getByTestId("groups-save-alice")).toBeInTheDocument();
});

test("create group from modal with user as member", async () => {
  const fetchMock = vi.fn(() =>
    Promise.resolve({
      ok: true,
      json: () => Promise.resolve({}),
    } as unknown as Response),
  );
  globalThis.fetch = fetchMock;
  const user = { ...alice, groups: [] };

  render(UserModal, { props: { user, onClose: () => {} } });
  await fireEvent.input(screen.getByTestId("groups-new-input-alice"), {
    target: { value: "devs" },
  });
  await fireEvent.keyDown(screen.getByTestId("groups-new-input-alice"), {
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
    } as unknown as Response),
  );
  const user = { ...alice, groups: [] };

  renderWithToaster(UserModal, { user, onClose: () => {} });
  await fireEvent.input(screen.getByTestId("groups-new-input-alice"), {
    target: { value: "devs" },
  });
  await fireEvent.keyDown(screen.getByTestId("groups-new-input-alice"), {
    key: "Enter",
  });

  const error = await screen.findByTestId("toast-error");

  expect(error).toHaveTextContent("ldap: boom");
  expect(screen.getByTestId("groups-new-input-alice")).toBeInTheDocument();
});
