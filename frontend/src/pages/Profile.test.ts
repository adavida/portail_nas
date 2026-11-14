import { fireEvent, screen, waitFor } from "@testing-library/svelte";
import { expect, test } from "vitest";
import { mockFetchByUrl } from "../test/mock_fetch_by_url";
import { renderWithToaster } from "../test/render_with_toaster";
import Profile from "./Profile.svelte";

const me = {
  sub: "uuid",
  preferred_username: "alice",
  name: "Alice",
  email: "a@ex.com",
  groups: [],
  is_admin: false,
};

const config = {
  issuer: "http://x",
  client_id: "c",
  redirect_uri: "r",
  password_min_length: 8,
};

test("pre-fills name and email from /api/auth/me", async () => {
  globalThis.fetch = mockFetchByUrl({
    "/api/auth/me": me,
    "/api/auth/config": config,
  });

  renderWithToaster(Profile, {});

  const form = await screen.findByTestId("profile-form");
  const name = screen.getByTestId("profile-name");
  const email = screen.getByTestId("profile-email");

  expect(form).toBeInTheDocument();
  expect(name).toHaveValue("Alice");
  expect(email).toHaveValue("a@ex.com");
  expect(screen.getByTestId("profile-save")).toBeDisabled();
});

test("tabs switch between informations and password", async () => {
  globalThis.fetch = mockFetchByUrl({
    "/api/auth/me": me,
    "/api/auth/config": config,
  });

  renderWithToaster(Profile, {});

  await screen.findByTestId("profile-form");
  expect(screen.queryByTestId("profile-password")).not.toBeInTheDocument();

  await fireEvent.click(screen.getByTestId("profile-tab-password"));

  const rule = await screen.findByText("At least 8 characters");
  expect(screen.getByTestId("profile-password")).toBeInTheDocument();
  expect(screen.queryByTestId("profile-name")).not.toBeInTheDocument();
  expect(rule).toBeInTheDocument();

  await fireEvent.click(screen.getByTestId("profile-tab-infos"));

  expect(screen.getByTestId("profile-name")).toBeInTheDocument();
  expect(screen.queryByTestId("profile-password")).not.toBeInTheDocument();
});

test("save sends PUT /api/auth/me with current values", async () => {
  const fetchMock = mockFetchByUrl({
    "/api/auth/me": me,
    "/api/auth/config": config,
  });
  globalThis.fetch = fetchMock;

  renderWithToaster(Profile, {});

  await screen.findByTestId("profile-form");
  await fireEvent.input(screen.getByTestId("profile-name"), {
    target: { value: "Alice Dupont" },
  });
  await fireEvent.click(screen.getByTestId("profile-save"));

  await waitFor(() => {
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/auth/me",
      expect.objectContaining({
        method: "PUT",
        body: JSON.stringify({ name: "Alice Dupont", email: "a@ex.com" }),
      }),
    );
  });
});

test("password change sends PUT /api/auth/me/password", async () => {
  const fetchMock = mockFetchByUrl({
    "/api/auth/me": me,
    "/api/auth/config": config,
  });
  globalThis.fetch = fetchMock;

  renderWithToaster(Profile, {});

  await screen.findByTestId("profile-form");
  await fireEvent.click(screen.getByTestId("profile-tab-password"));
  await fireEvent.input(screen.getByTestId("profile-password"), {
    target: { value: "secret123" },
  });
  await fireEvent.input(screen.getByTestId("profile-password-confirm"), {
    target: { value: "secret123" },
  });
  await fireEvent.click(screen.getByTestId("profile-password-save"));

  await waitFor(() => {
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/auth/me/password",
      expect.objectContaining({
        method: "PUT",
        body: JSON.stringify({ password: "secret123" }),
      }),
    );
  });
});

test("too short or mismatched password keeps the button disabled", async () => {
  const fetchMock = mockFetchByUrl({
    "/api/auth/me": me,
    "/api/auth/config": config,
  });
  globalThis.fetch = fetchMock;

  renderWithToaster(Profile, {});

  await screen.findByTestId("profile-form");
  await fireEvent.click(screen.getByTestId("profile-tab-password"));
  const short = screen.getByText("At least 8 characters");

  await fireEvent.input(screen.getByTestId("profile-password"), {
    target: { value: "abc" },
  });
  await fireEvent.input(screen.getByTestId("profile-password-confirm"), {
    target: { value: "abc" },
  });

  expect(screen.getByTestId("profile-password-save")).toBeDisabled();
  expect(short).not.toHaveClass("ok");

  await fireEvent.input(screen.getByTestId("profile-password"), {
    target: { value: "secret123" },
  });

  expect(short).toHaveClass("ok");
  expect(screen.getByTestId("profile-password-save")).toBeDisabled();

  await fireEvent.input(screen.getByTestId("profile-password-confirm"), {
    target: { value: "secret123" },
  });

  expect(screen.getByTestId("profile-password-save")).toBeEnabled();
  expect(
    fetchMock.mock.calls.filter(([url]) => url === "/api/auth/me/password"),
  ).toHaveLength(0);
});
