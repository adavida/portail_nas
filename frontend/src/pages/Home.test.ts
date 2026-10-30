import { render, screen } from "@testing-library/svelte";
import { expect, test, vi } from "vitest";
import { mockFetchByUrl } from "../test/mock_fetch_by_url";
import Home from "./Home.svelte";

test("loading then apps list", async () => {
  globalThis.fetch = mockFetchByUrl({
    "/apps.json": [
      {
        name: "Nextcloud",
        url: "https://cloud.example/",
        description: "Files",
        icon: null,
      },
    ],
  });

  render(Home);

  expect(screen.getByTestId("apps-loading")).toBeInTheDocument();

  const link = await screen.findByTestId("app-link-0");

  expect(link).toHaveTextContent("Nextcloud");
  expect(link).toHaveAttribute("href", "https://cloud.example/");
});

test("empty shows message", async () => {
  globalThis.fetch = mockFetchByUrl({ "/apps.json": [] });

  render(Home);

  const empty = await screen.findByTestId("apps-empty");

  expect(empty).toHaveTextContent("No application");
});

test("error on fetch failure", async () => {
  globalThis.fetch = vi.fn(() => Promise.reject(new Error("fail")));

  render(Home);

  const error = await screen.findByTestId("apps-error");

  expect(error).toHaveTextContent("Error");
});

test("hides app whose required_group the user is not a member of", async () => {
  globalThis.fetch = mockFetchByUrl({
    "/apps.json": [
      { name: "Open", url: "https://open.example/" },
      {
        name: "Vpn",
        url: "https://vpn.example/",
        required_group: "vpn",
      },
    ],
  });

  render(Home, { props: { groups: ["devs"] } });

  const link = await screen.findByTestId("app-link-0");

  expect(link).toHaveTextContent("Open");
  expect(screen.queryByText("Vpn")).not.toBeInTheDocument();
});

test("shows app whose required_group the user is a member of", async () => {
  globalThis.fetch = mockFetchByUrl({
    "/apps.json": [
      { name: "Vpn", url: "https://vpn.example/", required_group: "vpn" },
    ],
  });

  render(Home, { props: { groups: ["vpn"] } });

  const link = await screen.findByTestId("app-link-0");

  expect(link).toHaveTextContent("Vpn");
});

test("hides app whose forbidden_group the user is a member of", async () => {
  globalThis.fetch = mockFetchByUrl({
    "/apps.json": [
      {
        name: "Guest portal",
        url: "https://guest.example/",
        forbidden_group: "staff",
      },
    ],
  });

  render(Home, { props: { groups: ["staff"] } });

  await screen.findByTestId("apps-empty");

  expect(screen.queryByText("Guest portal")).not.toBeInTheDocument();
});
