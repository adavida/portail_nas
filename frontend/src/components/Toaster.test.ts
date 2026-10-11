import { render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import { afterEach, expect, test, vi } from "vitest";
import Toaster from "./Toaster.svelte";
import { toast } from "./toast";

afterEach(() => {
  vi.useRealTimers();
});

test("shows toast in bottom left box", async () => {
  render(Toaster);

  toast("ldap: boom", true);
  await tick();

  expect(screen.getByTestId("toaster")).toBeInTheDocument();
  expect(screen.getByTestId("toast-error")).toHaveTextContent("ldap: boom");
});

test("auto dismisses after 15s", async () => {
  vi.useFakeTimers();
  render(Toaster);

  toast("hello");
  await tick();

  expect(screen.getByTestId("toast")).toBeInTheDocument();

  vi.advanceTimersByTime(15000);
  await tick();

  expect(screen.queryByTestId("toaster")).not.toBeInTheDocument();
});
