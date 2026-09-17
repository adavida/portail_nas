import { render, screen } from "@testing-library/react";
import App from "./App";

test("renders title", () => {
  render(<App />);
  expect(screen.getByText("Portail LDAP")).toBeInTheDocument();
});

test("shows backend status", async () => {
  global.fetch = vi.fn(() =>
    Promise.resolve({
      json: () => Promise.resolve({ status: "ok" }),
    } as Response),
  );
  render(<App />);
  expect(await screen.findByTestId("health")).toHaveTextContent("backend: ok");
});
