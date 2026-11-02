import { render, screen } from "@testing-library/svelte";
import { expect, test } from "vitest";
import ThemeLangControls from "./ThemeLangControls.svelte";

test("renders the theme toggle and the lang select", () => {
  render(ThemeLangControls);

  expect(screen.getByTestId("theme-toggle")).toBeInTheDocument();
  expect(screen.getByTestId("lang-select")).toBeInTheDocument();
});
