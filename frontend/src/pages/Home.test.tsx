import { render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import { mockFetchByUrl } from "../test/mock_fetch_by_url";
import Home from "./Home";

test("loading then apps list", async () => {
  globalThis.fetch = mockFetchByUrl({
    "/apps.json": [
      {
        name: "Authelia",
        url: "https://127.0.0.1:9091/",
        description: "SSO",
        icon: null,
      },
    ],
  });

  render(<Home />);

  expect(screen.getByTestId("apps-loading")).toBeInTheDocument();

  const link = await screen.findByTestId("app-link-0");

  expect(link).toHaveTextContent("Authelia");
  expect(link).toHaveAttribute("href", "https://127.0.0.1:9091/");
});

test("empty shows message", async () => {
  globalThis.fetch = mockFetchByUrl({ "/apps.json": [] });

  render(<Home />);

  const empty = await screen.findByTestId("apps-empty");

  expect(empty).toHaveTextContent("Aucune application");
});

test("error on fetch failure", async () => {
  globalThis.fetch = vi.fn(() => Promise.reject(new Error("fail")));

  render(<Home />);

  const error = await screen.findByTestId("apps-error");

  expect(error).toHaveTextContent("Erreur");
});
