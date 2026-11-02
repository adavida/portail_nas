import { fireEvent, render, screen } from "@testing-library/svelte";
import { get } from "svelte/store";
import { afterEach, expect, test } from "vitest";
import { lang } from "../i18n";
import LangSelect from "./LangSelect.svelte";

afterEach(() => {
  lang.set("en");
});

test("selecting a language updates the lang store", () => {
  render(LangSelect);

  fireEvent.change(screen.getByTestId("lang-select"), {
    target: { value: "fr" },
  });

  expect(get(lang)).toBe("fr");
});
