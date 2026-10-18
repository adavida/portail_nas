import { render, screen } from "@testing-library/svelte";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import Callback from "./Callback.svelte";

const mocks = vi.hoisted(() => ({
  handleCallback: vi.fn<() => Promise<void>>(),
}));

vi.mock("../auth/oidc", () => ({ handleCallback: mocks.handleCallback }));

const originalLocation = window.location;

// jsdom blocks navigation — stub `location` (same pattern as Login.test.ts):
// `search` feeds ?code=&state=, `replace` captures the redirect home.
function stubLocation(search: string) {
  Object.defineProperty(window, "location", {
    configurable: true,
    value: { search, replace: vi.fn() },
  });
}

beforeEach(() => {
  sessionStorage.clear();
  localStorage.clear();
  mocks.handleCallback.mockReset().mockResolvedValue(undefined);
});

afterEach(() => {
  Object.defineProperty(window, "location", {
    configurable: true,
    value: originalLocation,
  });
});

test("exchanges the code then redirects home", async () => {
  stubLocation("?code=c&state=s");
  sessionStorage.setItem("oidc_state", "s");

  render(Callback);

  await vi.waitFor(() => {
    expect(window.location.replace).toHaveBeenCalledWith("/");
  });
  expect(mocks.handleCallback).toHaveBeenCalledWith("c");
  expect(sessionStorage.getItem("oidc_state")).toBeNull();
});

test("state mismatch shows the error and never exchanges", () => {
  stubLocation("?code=c&state=other");
  sessionStorage.setItem("oidc_state", "s");

  render(Callback);

  expect(screen.getByTestId("callback-error")).toHaveTextContent(
    "state mismatch",
  );
  expect(mocks.handleCallback).not.toHaveBeenCalled();
});

test("missing code shows the provider error", () => {
  stubLocation("?error=access_denied&error_description=user+cancelled");

  render(Callback);

  expect(screen.getByTestId("callback-error")).toHaveTextContent(
    "access_denied: user cancelled",
  );
});

test("an already exchanged code is ignored (reload guard)", async () => {
  stubLocation("?code=c");
  sessionStorage.setItem("handled_c", "1");

  render(Callback);

  expect(mocks.handleCallback).not.toHaveBeenCalled();
  expect(window.location.replace).not.toHaveBeenCalled();
});
