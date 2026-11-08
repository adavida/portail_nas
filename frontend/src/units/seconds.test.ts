import { expect, test } from "vitest";
import { Seconds } from "./seconds";

test("seconds normalizes ms and s labels to seconds", () => {
  expect(new Seconds("250 ms").value).toBeCloseTo(0.25);
  expect(new Seconds("3600.4 s").value).toBeCloseTo(3600.4);
});

test("seconds keeps the wire text", () => {
  expect(String(new Seconds("1 s"))).toBe("1 s");
});
