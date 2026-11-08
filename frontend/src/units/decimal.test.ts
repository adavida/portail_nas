import { expect, test } from "vitest";
import { Percent } from "./percent";
import { Celsius } from "./celsius";
import { Hundredths } from "./hundredths";
import { Rpm } from "./rpm";

test("percent parses a trailing signed label", () => {
  expect(new Percent("42 %").value).toBe(42);
  expect(String(new Percent("42 %"))).toBe("42 %");
});

test("celsius parses signed decimals", () => {
  expect(new Celsius("45.5 °C").value).toBeCloseTo(45.5);
  expect(new Celsius("-5.5 °C").value).toBeCloseTo(-5.5);
});

test("hundredths parses bare decimals", () => {
  expect(new Hundredths("2.50").value).toBeCloseTo(2.5);
  expect(String(new Hundredths("0.30"))).toBe("0.30");
});

test("rpm parses the sensor value", () => {
  expect(new Rpm("1200 rpm").value).toBe(1200);
});
