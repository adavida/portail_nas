import { render, screen } from "@testing-library/svelte";
import { expect, test } from "vitest";
import Gauge from "./Gauge.svelte";

test("draws the needle at half scale for 50 percent", () => {
  render(Gauge, {
    props: { label: "/data", percent: 50, detail: "1 GB / 2 GB" },
  });

  const needle = screen.getByTestId("gauge-needle");

  expect(needle.getAttribute("x2")).toBe("50.00");
  expect(needle.getAttribute("y2")).toBe("10.00");
});

test("clamps values above 100 to the right stop", () => {
  render(Gauge, { props: { label: "/data", percent: 140 } });

  const needle = screen.getByTestId("gauge-needle");

  expect(needle.getAttribute("x2")).toBe("90.00");
  expect(needle.getAttribute("y2")).toBe("50.00");
});

test("turns the fill red inside the red zone", () => {
  render(Gauge, { props: { label: "/data", percent: 95 } });

  const fill = screen.getByTestId("gauge-fill");

  expect(fill.classList.contains("hot")).toBe(true);
});

test("shows label and detail below the dial", () => {
  render(Gauge, {
    props: { label: "Disque /data", percent: 42.6, detail: "3 GB / 7 GB" },
  });

  const gauge = screen.getByTestId("gauge");

  expect(gauge).toHaveTextContent("Disque /data");
  expect(gauge).toHaveTextContent("3 GB / 7 GB");
  expect(screen.queryByText("42.6%")).toBeNull();
});
