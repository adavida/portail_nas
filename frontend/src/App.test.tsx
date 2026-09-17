import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";
import App from "./App";

beforeEach(() => {
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({
      ok: true,
      json: () => Promise.resolve([]),
    } as unknown as Response),
  );
});

test("renders users tab by default", async () => {
  render(<App />);

  await waitFor(() => {
    expect(screen.getByTestId("users-table")).toBeInTheDocument();
  });

  expect(screen.queryByTestId("groups-table")).not.toBeInTheDocument();
});

test("switches to groups tab", async () => {
  render(<App />);

  const groupsTab = screen.getByTestId("tab-groups");
  fireEvent.click(groupsTab);

  await waitFor(() => {
    expect(screen.getByTestId("groups-table")).toBeInTheDocument();
  });

  expect(screen.queryByTestId("users-table")).not.toBeInTheDocument();
});
