import { render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import Groups from "./Groups";

test("loading then data", async () => {
  const mockGroups = [{ gid: "devs", name: "Devs", description: "Devs team" }];
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({
      ok: true,
      json: () => Promise.resolve(mockGroups),
    } as Response),
  );

  render(<Groups />);

  expect(screen.getByTestId("groups-loading")).toBeInTheDocument();

  const table = await screen.findByTestId("groups-table");
  const row = screen.getByTestId("group-row-devs");

  expect(table).toBeInTheDocument();
  expect(row).toHaveTextContent("Devs team");
});

test("empty shows message", async () => {
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({ ok: true, json: () => Promise.resolve([]) } as Response),
  );

  render(<Groups />);

  const empty = await screen.findByTestId("groups-empty");

  expect(empty).toHaveTextContent("Aucun groupe");
});

test("error", async () => {
  globalThis.fetch = vi.fn(() => Promise.reject(new Error("fail")));

  render(<Groups />);

  const error = await screen.findByTestId("groups-error");

  expect(error).toHaveTextContent("Erreur");
});
