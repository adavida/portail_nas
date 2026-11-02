import { fireEvent, render, screen } from "@testing-library/svelte";
import { expect, test, vi } from "vitest";
import EditableCell from "./EditableCell.svelte";

type SaveFn = (field: string, value: string) => Promise<boolean>;

async function openEditor(onSave: SaveFn) {
  render(EditableCell, {
    props: { value: "alice", field: "name", rowId: "u1", onSave },
  });

  await fireEvent.click(screen.getByTestId("cell-name-u1"));

  return screen.getByTestId("edit-input-name-u1") as HTMLInputElement;
}

test("clicking the cell swaps it for an input prefilled with the value", async () => {
  const cell = render(EditableCell, {
    props: { value: "alice", field: "name", rowId: "u1", onSave: vi.fn() },
  }).getByTestId("cell-name-u1");

  expect(cell).toHaveTextContent("alice");

  await fireEvent.click(cell);

  expect(screen.getByTestId("edit-input-name-u1")).toHaveValue("alice");
  expect(screen.queryByTestId("cell-name-u1")).not.toBeInTheDocument();
});

test("Enter saves the draft and closes the editor on success", async () => {
  const onSave = vi.fn(async () => true);

  const input = await openEditor(onSave);
  await fireEvent.input(input, { target: { value: "bob" } });
  await fireEvent.keyDown(input, { key: "Enter" });

  expect(onSave).toHaveBeenCalledWith("name", "bob");
  await vi.waitFor(() => {
    expect(screen.getByTestId("cell-name-u1")).toBeInTheDocument();
  });
});

test("Enter keeps the editor open when the save fails", async () => {
  const onSave = vi.fn(async () => false);

  const input = await openEditor(onSave);
  await fireEvent.input(input, { target: { value: "bob" } });
  await fireEvent.keyDown(input, { key: "Enter" });

  expect(onSave).toHaveBeenCalledWith("name", "bob");
  expect(screen.getByTestId("edit-input-name-u1")).toBeInTheDocument();
});

test("Escape cancels without calling onSave", async () => {
  const onSave = vi.fn(async () => true);

  const input = await openEditor(onSave);
  await fireEvent.keyDown(input, { key: "Escape" });

  expect(onSave).not.toHaveBeenCalled();
  expect(screen.getByTestId("cell-name-u1")).toBeInTheDocument();
});
