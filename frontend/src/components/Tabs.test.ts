import { fireEvent, render, screen } from "@testing-library/svelte";
import { expect, test } from "vitest";
import Tabs from "./Tabs.svelte";

const tabs = [
  { id: "infos", label: "Informations", testId: "tab-infos" },
  { id: "password", label: "Mot de passe", testId: "tab-password" },
];

test("renders tabs and marks the active one", () => {
  render(Tabs, { props: { tabs, active: "infos" } });

  const infos = screen.getByTestId("tab-infos");
  const password = screen.getByTestId("tab-password");

  expect(infos).toHaveAttribute("aria-selected", "true");
  expect(infos).toHaveClass("active");
  expect(password).toHaveAttribute("aria-selected", "false");
});

test("clicking a tab makes it active", async () => {
  render(Tabs, { props: { tabs, active: "infos" } });

  await fireEvent.click(screen.getByTestId("tab-password"));

  expect(screen.getByTestId("tab-password")).toHaveAttribute(
    "aria-selected",
    "true",
  );
  expect(screen.getByTestId("tab-infos")).toHaveAttribute(
    "aria-selected",
    "false",
  );
});
