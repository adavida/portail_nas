import { expect, test } from "vitest";
import { BitsPerSecond } from "./bits_per_second";

test("bits per second scales decimal link units", () => {
  expect(new BitsPerSecond("100 Mib/s").value).toBe(100 * 10 ** 6);
  expect(new BitsPerSecond("1 Gib/s").value).toBe(10 ** 9);
  expect(new BitsPerSecond("0 bit/s").value).toBe(0);
});

test("bits per second keeps the wire text", () => {
  expect(String(new BitsPerSecond("1 Gib/s"))).toBe("1 Gib/s");
});
