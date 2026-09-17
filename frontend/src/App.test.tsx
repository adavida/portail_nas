import { render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import App from "./App";

test("renders title", () => {
  render(<App />);

  expect(screen.getByText("Utilisateurs")).toBeInTheDocument();
});
