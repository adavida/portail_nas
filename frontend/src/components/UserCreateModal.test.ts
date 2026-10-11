import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { expect, test, vi } from "vitest";
import { renderWithToaster } from "../test/render_with_toaster";
import UserCreateModal from "./UserCreateModal.svelte";

test("submit creates user, refreshes and closes", async () => {
  const fetchMock = vi.fn(() =>
    Promise.resolve({
      ok: true,
      json: () => Promise.resolve({}),
    } as unknown as Response),
  );
  globalThis.fetch = fetchMock;
  const onCreated = vi.fn();
  const onClose = vi.fn();

  render(UserCreateModal, { props: { onClose, onCreated } });
  await fireEvent.input(screen.getByTestId("input-uid"), {
    target: { value: "carol" },
  });
  await fireEvent.input(screen.getByTestId("input-name"), {
    target: { value: "Carol" },
  });
  await fireEvent.input(screen.getByTestId("input-email"), {
    target: { value: "carol@example.com" },
  });
  await fireEvent.input(screen.getByTestId("input-password"), {
    target: { value: "s3cret" },
  });
  await fireEvent.click(screen.getByTestId("create-button"));

  await waitFor(() => {
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/users",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({
          uid: "carol",
          name: "Carol",
          email: "carol@example.com",
          password: "s3cret",
        }),
      }),
    );
    expect(onCreated).toHaveBeenCalled();
    expect(onClose).toHaveBeenCalled();
  });
});

test("missing fields asks for uid, name and password", async () => {
  const fetchMock = vi.fn();
  globalThis.fetch = fetchMock;

  renderWithToaster(UserCreateModal, { onClose: () => {} });
  await fireEvent.input(screen.getByTestId("input-uid"), {
    target: { value: "carol" },
  });
  await fireEvent.click(screen.getByTestId("create-button"));

  const error = await screen.findByTestId("toast-error");

  expect(error).toHaveTextContent("uid, name and password required");
  expect(fetchMock).not.toHaveBeenCalled();
});

test("api error keeps modal open with message", async () => {
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({
      ok: false,
      status: 500,
      json: () => Promise.resolve({ error: "ldap: boom" }),
    } as unknown as Response),
  );

  renderWithToaster(UserCreateModal, { onClose: () => {} });
  await fireEvent.input(screen.getByTestId("input-uid"), {
    target: { value: "carol" },
  });
  await fireEvent.input(screen.getByTestId("input-name"), {
    target: { value: "Carol" },
  });
  await fireEvent.input(screen.getByTestId("input-password"), {
    target: { value: "s3cret" },
  });
  await fireEvent.click(screen.getByTestId("create-button"));

  const error = await screen.findByTestId("toast-error");

  expect(error).toHaveTextContent("ldap: boom");
  expect(screen.getByTestId("user-create-modal")).toBeInTheDocument();
});

test("cancel closes without calling api", async () => {
  const fetchMock = vi.fn();
  globalThis.fetch = fetchMock;
  const onClose = vi.fn();

  render(UserCreateModal, { props: { onClose } });
  await fireEvent.click(screen.getByTestId("create-cancel"));

  expect(onClose).toHaveBeenCalled();
  expect(fetchMock).not.toHaveBeenCalled();
});
