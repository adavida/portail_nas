import { render, screen } from "@testing-library/svelte";
import { expect, test } from "vitest";
import LabeledInput from "./LabeledInput.svelte";

test("renders label and input linked on one line", () => {
  render(LabeledInput, {
    props: { label: "Nom", value: "Alice", testId: "li-name" },
  });

  const input = screen.getByTestId("li-name");

  expect(screen.getByText("Nom")).toBeInTheDocument();
  expect(input.closest("label")).toHaveTextContent("Nom");
  expect(input).toHaveValue("Alice");
});

test("passes type and placeholder through", () => {
  render(LabeledInput, {
    props: {
      label: "Email",
      value: "",
      testId: "li-email",
      type: "email",
      placeholder: "a@b.c",
    },
  });

  const input = screen.getByTestId("li-email");

  expect(input).toHaveAttribute("type", "email");
  expect(input).toHaveAttribute("placeholder", "a@b.c");
});
