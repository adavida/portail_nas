import { writable } from "svelte/store";

type Theme = "light" | "dark";

function initialTheme(): Theme {
  const saved = localStorage.getItem("theme");
  if (saved === "light" || saved === "dark") return saved;
  return window.matchMedia?.("(prefers-color-scheme: dark)")?.matches
    ? "dark"
    : "light";
}

export const theme = writable<Theme>(initialTheme());

theme.subscribe((value) => {
  document.documentElement.dataset.theme = value;
  localStorage.setItem("theme", value);
});
