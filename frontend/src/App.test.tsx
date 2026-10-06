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

test("renders home by default for admin", async () => {
  render(
    <BrowserRouter>
      <App />
    </BrowserRouter>,
  );

  await waitFor(() => {
    expect(screen.getByTestId("apps-empty")).toBeInTheDocument();
  });

  expect(screen.getByTestId("tab-users")).toBeInTheDocument();
  expect(screen.queryByTestId("users-table")).not.toBeInTheDocument();
});

test("switches to users tab", async () => {
  render(
    <BrowserRouter>
      <App />
    </BrowserRouter>,
  );

  await waitFor(() => {
    expect(screen.getByTestId("apps-empty")).toBeInTheDocument();
  });

  fireEvent.click(screen.getByTestId("tab-users"));

  const table = await screen.findByTestId("users-table");

  expect(table).toBeInTheDocument();
});

test("switches to groups tab", async () => {
  render(
    <BrowserRouter>
      <App />
    </BrowserRouter>,
  );

  await waitFor(() => {
    expect(screen.getByTestId("apps-empty")).toBeInTheDocument();
  });

  fireEvent.click(screen.getByTestId("tab-groups"));

  const table = await screen.findByTestId("groups-table");

  expect(table).toBeInTheDocument();
});

test("stays logged out after manual logout", async () => {
  localStorage.clear();
  sessionStorage.clear();
  sessionStorage.setItem("logged_out", "1");

  render(
    <BrowserRouter>
      <App />
    </BrowserRouter>,
  );

  expect(await screen.findByTestId("logged-out")).toBeInTheDocument();

  const urls = (globalThis.fetch as ReturnType<typeof vi.fn>).mock.calls.map(
    (c) => String(c[0]),
  );
  expect(urls.some((u) => u.includes("/api/auth/config"))).toBe(false);
});

test("non-admin sees home only", async () => {
  globalThis.fetch = vi.fn((url: unknown) => {
    const u = String(url);
    if (u.includes("/api/auth/me"))
      return Promise.resolve({
        ok: false,
        status: 403,
      } as unknown as Response);
    return Promise.resolve({
      ok: true,
      json: () => Promise.resolve([]),
    } as unknown as Response);
  });

  render(
    <BrowserRouter>
      <App />
    </BrowserRouter>,
  );

  await waitFor(() => {
    expect(screen.getByTestId("apps-empty")).toBeInTheDocument();
  });

  expect(screen.queryByTestId("tab-users")).not.toBeInTheDocument();
  expect(screen.queryByTestId("tab-groups")).not.toBeInTheDocument();
});
