import { fireEvent, render, screen } from "@testing-library/svelte";
import { get } from "svelte/store";
import { afterEach, expect, test } from "vitest";
import { theme } from "../theme";
import ThemeToggle from "./ThemeToggle.svelte";

afterEach(() => {
  document.documentElement.removeAttribute("data-theme");
  localStorage.removeItem("theme");
});

test("clicking the toggle flips the theme and persists it", () => {
  render(ThemeToggle);

  const toggle = screen.getByTestId("theme-toggle");
  const initial = get(theme);
  fireEvent.click(toggle);

  expect(get(theme)).toBe(initial === "dark" ? "light" : "dark");
  expect(toggle).toHaveAttribute("aria-checked", String(get(theme) === "dark"));
  expect(document.documentElement.dataset.theme).toBe(get(theme));
  expect(localStorage.getItem("theme")).toBe(get(theme));
});
