import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";
import { BrowserRouter } from "react-router-dom";
import App from "./App";

beforeEach(() => {
  localStorage.setItem("access_token", "test-token");
  globalThis.fetch = vi.fn((url: unknown) =>
    Promise.resolve({
      ok: true,
      json: () => {
        if (String(url).includes("/api/auth/config"))
          return Promise.resolve({
            issuer: "https://127.0.0.1:9091",
            client_id: "portail-dev",
            redirect_uri: "http://localhost:5173/callback",
          });
        return Promise.resolve([]);
      },
    } as unknown as Response),
  );
});

test("renders users tab by default", async () => {
  render(
    <BrowserRouter>
      <App />
    </BrowserRouter>,
  );

  await waitFor(() => {
    expect(screen.getByTestId("users-table")).toBeInTheDocument();
  });

  expect(screen.queryByTestId("groups-table")).not.toBeInTheDocument();
});

test("switches to groups tab", async () => {
  render(
    <BrowserRouter>
      <App />
    </BrowserRouter>,
  );

  const groupsTab = screen.getByTestId("tab-groups");
  fireEvent.click(groupsTab);

  await waitFor(() => {
    expect(screen.getByTestId("groups-table")).toBeInTheDocument();
  });

  expect(screen.queryByTestId("users-table")).not.toBeInTheDocument();
});
