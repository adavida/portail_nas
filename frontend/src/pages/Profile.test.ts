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
  expect(screen.queryByTestId("profile-attrs")).not.toBeInTheDocument();
});

test("attr fields show the localized name of the ui language", async () => {
  globalThis.fetch = mockFetchByUrl({
    "/api/auth/me": {
      ...me,
      attrs: [
        {
          label: "city",
          value: "",
          type: "text",
          max_chars: null,
          min: null,
          max: null,
          names: { fr: "Ville", en: "City" },
        },
      ],
    },
    "/api/auth/config": config,
  });

  renderWithToaster(Profile, {});

  const field = await screen.findByTestId("profile-attr-city");

  expect(field.closest("label")).toHaveTextContent("City");
});

test("typing a custom attr enables the confirm button", async () => {
  globalThis.fetch = mockFetchByUrl({
    "/api/auth/me": {
      ...me,
      attrs: [
        {
          label: "telephone",
          value: "",
          type: "telephone",
          max_chars: null,
          min: null,
          max: null,
        },
      ],
    },
    "/api/auth/config": config,
  });

  renderWithToaster(Profile, {});

  const phone = await screen.findByTestId("profile-attr-telephone");
  const save = screen.getByTestId("profile-save");
  expect(save).toBeDisabled();

  await fireEvent.input(phone, { target: { value: "+33 6 12 34 56 78" } });

  expect(save).not.toBeDisabled();
});

test("phone attrs are sent without spaces (E.164)", async () => {
  const fetchMock = mockFetchByUrl({
    "/api/auth/me": {
      ...me,
      attrs: [
        {
          label: "telephone",
          value: "",
          type: "telephone",
          max_chars: null,
          min: null,
          max: null,
        },
      ],
    },
    "PUT /api/auth/me": { profile_saved: true, attrs_saved: true },
    "/api/auth/config": config,
  });
  globalThis.fetch = fetchMock;

  renderWithToaster(Profile, {});

  const phone = await screen.findByTestId("profile-attr-telephone");
  expect(phone).toHaveAttribute("type", "tel");

  await fireEvent.input(phone, { target: { value: "0612345678" } });
  expect(screen.getByTestId("profile-attr-telephone-dial")).toHaveTextContent(
    "+33",
  );
  expect(phone).toHaveValue("6 12 34 56 78");

  await fireEvent.click(screen.getByTestId("profile-save"));

  await waitFor(() => {
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/auth/me",
      expect.objectContaining({
        method: "PUT",
        body: JSON.stringify({
          name: "Alice",
          email: "a@ex.com",
          attrs: [{ label: "telephone", value: "+33612345678" }],
        }),
      }),
    );
  });
  expect(
    fetchMock.mock.calls.filter(
      ([url, init]) => url === "/api/auth/me/attrs" && init?.method === "PUT",
    ),
  ).toHaveLength(0);
});

test("a failing profile half does not block saving attrs", async () => {
  const fetchMock = mockFetchByUrl({
    "/api/auth/me": {
      ...me,
      attrs: [
        {
          label: "city",
          value: "Paris",
          type: "text",
          max_chars: null,
          min: null,
          max: null,
        },
      ],
    },
    "PUT /api/auth/me": {
      profile_saved: false,
      attrs_saved: true,
      error: "name is empty",
    },
    "/api/auth/config": config,
  });
  globalThis.fetch = fetchMock;

  renderWithToaster(Profile, {});

  const city = await screen.findByTestId("profile-attr-city");
  await fireEvent.input(city, { target: { value: "" } });
  await fireEvent.click(screen.getByTestId("profile-save"));

  await waitFor(() => {
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/auth/me",
      expect.objectContaining({
        method: "PUT",
        body: JSON.stringify({
          name: "Alice",
          email: "a@ex.com",
          attrs: [{ label: "city", value: "" }],
        }),
      }),
    );
  });
});

test("custom attrs render prefilled and are saved with the form", async () => {
  const fetchMock = mockFetchByUrl({
    "/api/auth/me": {
      ...me,
      attrs: [
        {
          label: "city",
          value: "Paris",
          type: "text",
          max_chars: 50,
          min: null,
          max: null,
        },
        {
          label: "age",
          value: "30",
          type: "number",
          max_chars: null,
          min: 0,
          max: 150,
        },
      ],
    },
    "PUT /api/auth/me": { profile_saved: true, attrs_saved: true },
    "/api/auth/config": config,
  });
  globalThis.fetch = fetchMock;

  renderWithToaster(Profile, {});

  const city = await screen.findByTestId("profile-attr-city");
  const age = screen.getByTestId("profile-attr-age");

  expect(city).toHaveValue("Paris");
  expect(age).toHaveAttribute("type", "number");
  expect(screen.getByTestId("profile-save")).toBeDisabled();

  await fireEvent.input(city, { target: { value: "Lyon" } });
  await fireEvent.click(screen.getByTestId("profile-save"));

  await waitFor(() => {
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/auth/me",
      expect.objectContaining({
        method: "PUT",
        body: JSON.stringify({
          name: "Alice",
          email: "a@ex.com",
          attrs: [
            { label: "city", value: "Lyon" },
            { label: "age", value: "30" },
          ],
        }),
      }),
    );
  });
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
        body: JSON.stringify({
          name: "Alice Dupont",
          email: "a@ex.com",
          attrs: [],
        }),
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
