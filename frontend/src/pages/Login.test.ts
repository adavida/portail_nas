import { fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, expect, test, vi } from "vitest";
import Login from "./Login.svelte";

const originalLocation = window.location;

function stubLocation(search: string) {
  Object.defineProperty(window, "location", {
    configurable: true,
    value: { search, href: "" },
  });
}

function mockLoginResponse(status: number, payload: unknown) {
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({
      ok: status === 200,
      status,
      json: () => Promise.resolve(payload),
    } as unknown as Response),
  );
}

async function fillAndSubmit() {
  await fireEvent.input(screen.getByTestId("input-username"), {
    target: { value: "alice" },
  });
  await fireEvent.input(screen.getByTestId("input-password"), {
    target: { value: "secret" },
  });
  await fireEvent.click(screen.getByTestId("login-button"));
}

afterEach(() => {
  Object.defineProperty(window, "location", {
    configurable: true,
    value: originalLocation,
  });
});

test("shows theme and language controls in the corner", () => {
  stubLocation("?req=flow-1");

  render(Login);

  expect(screen.getByTestId("theme-toggle")).toBeInTheDocument();
  expect(screen.getByTestId("lang-select")).toBeInTheDocument();
});

test("200 redirects to the provider callback URL", async () => {
  stubLocation("?req=flow-1");
  mockLoginResponse(200, {
    redirect: "http://localhost:5173/callback?code=c&state=s",
  });

  render(Login);
  await fillAndSubmit();

  await vi.waitFor(() => {
    expect(window.location.href).toBe(
      "http://localhost:5173/callback?code=c&state=s",
    );
  });

  const body = JSON.parse(
    String(
      (globalThis.fetch as ReturnType<typeof vi.fn>).mock.calls[0][1].body,
    ),
  );
  expect(body).toEqual({
    req: "flow-1",
    username: "alice",
    password: "secret",
  });
});

test("401 shows translated credentials error and re-enables submit", async () => {
  stubLocation("?req=flow-1");
  mockLoginResponse(401, { error: "invalid username or password" });

  render(Login);
  await fillAndSubmit();

  const error = await screen.findByTestId("login-error");

  expect(error).toHaveTextContent("Invalid username or password");
  expect(error).toHaveAttribute("role", "alert");
  expect(screen.getByTestId("input-username")).toHaveAttribute(
    "aria-invalid",
    "true",
  );
  expect(screen.getByTestId("login-button")).toBeEnabled();
});

test("400 shows the generic backend error", async () => {
  stubLocation("?req=flow-1");
  mockLoginResponse(400, { error: "expired or unknown request" });

  render(Login);
  await fillAndSubmit();

  const error = await screen.findByTestId("login-error");

  expect(error).toHaveTextContent("expired or unknown request");
});

test("missing req shows error, never fetches, submit disabled", async () => {
  stubLocation("");
  globalThis.fetch = vi.fn();

  render(Login);
  await fillAndSubmit();

  const error = screen.getByTestId("login-error");

  expect(error).toHaveTextContent("Missing or expired login request");
  expect(globalThis.fetch).not.toHaveBeenCalled();
  expect(screen.getByTestId("login-button")).toBeDisabled();
});

test("submit is disabled while the request is in flight", async () => {
  stubLocation("?req=flow-1");
  let resolve!: (r: Response) => void;
  globalThis.fetch = vi.fn(() => new Promise<Response>((r) => (resolve = r)));

  render(Login);
  await fireEvent.input(screen.getByTestId("input-username"), {
    target: { value: "alice" },
  });
  await fireEvent.input(screen.getByTestId("input-password"), {
    target: { value: "secret" },
  });
  await fireEvent.click(screen.getByTestId("login-button"));

  expect(screen.getByTestId("login-button")).toBeDisabled();

  resolve({
    ok: true,
    status: 200,
    json: () => Promise.resolve({ redirect: "http://localhost:5173/callback" }),
  } as unknown as Response);
  await vi.waitFor(() => {
    expect(window.location.href).toBe("http://localhost:5173/callback");
  });
});
