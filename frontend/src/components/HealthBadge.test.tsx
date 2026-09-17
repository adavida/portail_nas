import { render, screen } from "@testing-library/react";
import { expect, test } from "vitest";
import { HealthBadge } from "./HealthBadge";

test("renders status", () => {
  render(<HealthBadge status="ok" />);
  expect(screen.getByTestId("health")).toHaveTextContent("backend: ok");
});
