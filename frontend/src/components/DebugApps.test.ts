import { render, screen } from "@testing-library/svelte";
import { expect, test } from "vitest";
import DebugApps from "./DebugApps.svelte";

const client = {
  id: "portail-dev",
  enabled: true,
  has_secret: true,
  redirect_uris: ["http://localhost:5173/callback"],
  scopes: ["openid", "groups"],
  required_group: "",
  forbidden_group: "",
};

test("merges app and client with the same id on one row", () => {
  render(DebugApps, {
    apps: [{ name: "portail-dev", url: "http://x.example/" }],
    clients: [client],
  });

  const row = screen.getByTestId("debug-app-0");

  expect(row).toHaveTextContent("http://x.example/");
  expect(row).toHaveTextContent("portail-dev");
  expect(screen.queryByTestId("debug-client-0")).not.toBeInTheDocument();
});

test("keeps unmatched apps and clients on separate rows", () => {
  render(DebugApps, {
    apps: [{ name: "Nextcloud", url: "https://cloud.example/" }],
    clients: [client],
  });

  expect(screen.getByTestId("debug-app-0")).toHaveTextContent("Nextcloud");
  expect(screen.getByTestId("debug-client-0")).toHaveTextContent("portail-dev");
});

test("shows the empty message when there is nothing to list", () => {
  render(DebugApps, { apps: [], clients: [] });

  expect(screen.getByTestId("debug-apps-empty")).toBeInTheDocument();
});
