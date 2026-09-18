import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";
import { BrowserRouter } from "react-router-dom";
import App from "./App";

beforeEach(() => {
  const adminPayload = btoa(JSON.stringify({ groups: ["admin"] }));
  localStorage.setItem("id_token", `eyJhbGciOiJIUzI1NiJ9.${adminPayload}.sig`);
  localStorage.setItem("access_token", "test-token");
  globalThis.fetch = vi.fn((url: unknown) => {
    const u = String(url);
    if (u.includes("/api/auth/config"))
      return Promise.resolve({
        ok: true,
        json: () =>
          Promise.resolve({
            issuer: "https://127.0.0.1:9091",
            client_id: "portail-dev",
            redirect_uri: "http://localhost:5173/callback",
          }),
      } as unknown as Response);
    if (u.includes("/api/auth/me"))
      return Promise.resolve({
        ok: true,
        json: () => Promise.resolve({ sub: "admin", groups: ["admin"] }),
      } as unknown as Response);
    return Promise.resolve({
      ok: true,
      json: () => Promise.resolve([]),
    } as unknown as Response);
  });
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

  await waitFor(() => {
    expect(screen.getByTestId("users-table")).toBeInTheDocument();
  });

  const groupsTab = screen.getByTestId("tab-groups");
  fireEvent.click(groupsTab);

  await waitFor(() => {
    expect(screen.getByTestId("groups-table")).toBeInTheDocument();
  });

  expect(screen.queryByTestId("users-table")).not.toBeInTheDocument();
});
