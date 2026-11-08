import { render, screen } from "@testing-library/svelte";
import { expect, test } from "vitest";
import UsageBar from "./UsageBar.svelte";

test("renders label and detail", () => {
  render(UsageBar, {
    props: { label: "RAM", percent: 42, detail: "6 GB / 16 GB" },
  });

  const bar = screen.getByTestId("usage-bar");

  expect(bar).toHaveTextContent("RAM");
  expect(bar).toHaveTextContent("6 GB / 16 GB");
});

test("clamps percent above 100 to full width", () => {
  render(UsageBar, { props: { label: "CPU", percent: 140 } });

  const fill = screen.getByTestId("usage-fill");

  expect(fill).toHaveStyle({ width: "100%" });
});

test("clamps negative percent to zero width", () => {
  render(UsageBar, { props: { label: "CPU", percent: -5 } });

  const fill = screen.getByTestId("usage-fill");

  expect(fill).toHaveStyle({ width: "0%" });
});
