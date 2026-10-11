import { fireEvent, render, screen } from "@testing-library/svelte";
import { expect, test, vi } from "vitest";
import FieldRow from "./FieldRow.svelte";

test("renders label and input, save and cancel run callbacks", async () => {
  const onValidate = vi.fn();
  const onCancel = vi.fn();

  render(FieldRow, {
    props: {
      label: "Nom",
      id: "f1",
      value: "Alice",
      onValidate,
      onCancel,
    },
  });

  expect(screen.getByText("Nom")).toBeInTheDocument();
  await fireEvent.input(screen.getByTestId("f1"), { target: { value: "B" } });
  await fireEvent.click(screen.getByTestId("f1-save"));

  expect(onValidate).toHaveBeenCalled();
  await fireEvent.click(screen.getByTestId("f1-cancel"));

  expect(onCancel).toHaveBeenCalled();
});
