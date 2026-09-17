import { render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import Home from "./Home";

test("renders title", () => {
  render(<Home />);

  expect(screen.getByText("Portail LDAP")).toBeInTheDocument();
});

test("shows backend status", async () => {
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({
      json: () => Promise.resolve({ status: "ok" }),
    } as Response),
  );

  render(<Home />);

  expect(await screen.findByTestId("health")).toHaveTextContent("backend: ok");
});
