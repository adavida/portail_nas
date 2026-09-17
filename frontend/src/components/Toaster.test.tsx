import { act, render, screen } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import { Toaster, toast } from "./Toaster";

afterEach(() => {
  vi.useRealTimers();
});

test("shows toast in bottom left box", () => {
  render(<Toaster />);

  act(() => toast("ldap: boom", true));

  expect(screen.getByTestId("toaster")).toBeInTheDocument();
  expect(screen.getByTestId("toast-error")).toHaveTextContent("ldap: boom");
});

test("auto dismisses after 15s", () => {
  vi.useFakeTimers();
  render(<Toaster />);

  act(() => toast("hello"));
  expect(screen.getByTestId("toast")).toBeInTheDocument();

  act(() => {
    vi.advanceTimersByTime(15000);
  });

  expect(screen.queryByTestId("toaster")).not.toBeInTheDocument();
});
