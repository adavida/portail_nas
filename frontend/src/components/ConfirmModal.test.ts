import { fireEvent, render, screen } from "@testing-library/svelte";
import { expect, test, vi } from "vitest";
import ConfirmModal from "./ConfirmModal.svelte";

test("confirm runs onConfirm", async () => {
  const onConfirm = vi.fn();
  const onCancel = vi.fn();

  render(ConfirmModal, {
    props: { message: "Supprimer alice ?", onConfirm, onCancel },
  });

  await fireEvent.click(screen.getByTestId("confirm-yes"));

  expect(onConfirm).toHaveBeenCalled();
  expect(onCancel).not.toHaveBeenCalled();
});

test("cancel runs onCancel and not onConfirm", async () => {
  const onConfirm = vi.fn();
  const onCancel = vi.fn();

  render(ConfirmModal, {
    props: { message: "Supprimer alice ?", onConfirm, onCancel },
  });

  await fireEvent.click(screen.getByTestId("confirm-no"));

  expect(onCancel).toHaveBeenCalled();
  expect(onConfirm).not.toHaveBeenCalled();
});
