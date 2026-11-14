import { fireEvent, render, screen } from "@testing-library/svelte";
import { expect, test, vi } from "vitest";
import Monitoring from "./Monitoring.svelte";

function mockAllFail() {
  return vi.fn(() =>
    Promise.resolve({
      ok: false,
      status: 500,
      json: () => Promise.resolve({ error: "down" }),
    } as unknown as Response),
  );
}

test("defaults to system then switches to debug", async () => {
  globalThis.fetch = mockAllFail();

  render(Monitoring);

  expect(await screen.findByTestId("system-error")).toBeInTheDocument();

  await fireEvent.click(screen.getByTestId("monitoring-tab-debug"));

  expect(await screen.findByTestId("debug-error")).toBeInTheDocument();
});
