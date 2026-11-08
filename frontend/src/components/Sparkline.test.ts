import { render, screen } from "@testing-library/svelte";
import { expect, test } from "vitest";
import Sparkline from "./Sparkline.svelte";

test("draws a polyline from percent points", () => {
  render(Sparkline, {
    props: { series: [{ label: "", points: [0, 50, 100] }] },
  });

  const line = screen.getByTestId("sparkline").querySelector("polyline");

  expect(line).not.toBeNull();
  expect(line?.getAttribute("points")).toBe(
    "0.00,100.00 50.00,50.00 100.00,0.00",
  );
});

test("clamps values above 100 to the top", () => {
  render(Sparkline, { props: { series: [{ label: "", points: [0, 140] }] } });

  const line = screen.getByTestId("sparkline").querySelector("polyline");

  expect(line?.getAttribute("points")).toBe("0.00,100.00 100.00,0.00");
});

test("renders nothing below two points", () => {
  render(Sparkline, { props: { series: [{ label: "", points: [42] }] } });

  const line = screen.getByTestId("sparkline").querySelector("polyline");

  expect(line).toBeNull();
});

test("scales points to a custom max", () => {
  render(Sparkline, {
    props: { series: [{ label: "", points: [0, 2] }], max: 4 },
  });

  const line = screen.getByTestId("sparkline").querySelector("polyline");

  expect(line?.getAttribute("points")).toBe("0.00,100.00 100.00,50.00");
});

test("draws a dashed reference line at its scaled height", () => {
  render(Sparkline, {
    props: { series: [{ label: "", points: [0, 2] }], max: 4, reference: 2 },
  });

  const reference = screen.getByTestId("sparkline-reference");

  expect(reference.getAttribute("y1")).toBe("50");
});

test("draws one line per series with a distinct color and a legend", () => {
  render(Sparkline, {
    props: {
      series: [
        { label: "RAM", points: [0, 50] },
        { label: "Swap", points: [50, 0] },
      ],
    },
  });

  const lines = screen.getByTestId("sparkline").querySelectorAll("polyline");
  const legend = screen.getByTestId("sparkline-legend");

  expect(lines).toHaveLength(2);
  expect(lines[1].getAttribute("stroke")).toBe("#f5a623");
  expect(legend).toHaveTextContent("RAM");
  expect(legend).toHaveTextContent("Swap");
});

test("hides the legend for a single series", () => {
  render(Sparkline, { props: { series: [{ label: "RAM", points: [0, 50] }] } });

  const legend = screen.queryByTestId("sparkline-legend");

  expect(legend).toBeNull();
});

test("shows the scale ticks with their unit on the side", () => {
  render(Sparkline, {
    props: { series: [{ label: "", points: [0, 50] }], unit: "%" },
  });

  const axis = screen.getByTestId("sparkline-axis");

  expect(axis).toHaveTextContent("100 %");
  expect(axis).toHaveTextContent("0");
});
