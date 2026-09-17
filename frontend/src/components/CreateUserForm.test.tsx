import { fireEvent, render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import { CreateUserForm } from "./CreateUserForm";

test("renders fields", () => {
  render(<CreateUserForm onCreated={() => {}} />);

  expect(screen.getByTestId("input-uid")).toBeInTheDocument();
  expect(screen.getByTestId("input-password")).toBeInTheDocument();
});

test("submits and clears", async () => {
  const onCreated = vi.fn();
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({ ok: true, json: () => Promise.resolve({}) } as Response),
  );

  render(<CreateUserForm onCreated={onCreated} />);

  fireEvent.change(screen.getByTestId("input-uid"), {
    target: { value: "bob" },
  });
  fireEvent.change(screen.getByTestId("input-name"), {
    target: { value: "Bob Dupont" },
  });
  fireEvent.change(screen.getByTestId("input-password"), {
    target: { value: "secret" },
  });
  fireEvent.click(screen.getByTestId("create-button"));

  await vi.waitFor(() => {
    expect(globalThis.fetch).toHaveBeenCalledWith(
      "/api/users",
      expect.objectContaining({ method: "POST" }),
    );
  });

  await vi.waitFor(() => {
    expect(onCreated).toHaveBeenCalled();
  });
});

test("shows error on 500", async () => {
  const errorResponse = { error: "uid exists" };
  globalThis.fetch = vi.fn(() =>
    Promise.resolve({
      ok: false,
      status: 500,
      json: () => Promise.resolve(errorResponse),
    } as Response),
  );

  render(<CreateUserForm onCreated={() => {}} />);

  fireEvent.change(screen.getByTestId("input-uid"), {
    target: { value: "bob" },
  });
  fireEvent.change(screen.getByTestId("input-name"), {
    target: { value: "Bob" },
  });
  fireEvent.change(screen.getByTestId("input-password"), {
    target: { value: "p" },
  });
  fireEvent.click(screen.getByTestId("create-button"));

  const error = await screen.findByTestId("create-error");

  expect(error).toHaveTextContent("uid exists");
});
