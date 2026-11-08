import { expect, test } from "vitest";
import { BytesPerSecond } from "./bytes_per_second";

test("bytes per second scales rate units to base", () => {
  expect(new BytesPerSecond("512 B/s").value).toBe(512);
  expect(new BytesPerSecond("2 KiB/s").value).toBe(2048);
  expect(new BytesPerSecond("10 MiB/s").value).toBe(10 * 1024 ** 2);
});

test("bytes per second keeps the wire text", () => {
  expect(String(new BytesPerSecond("10 MiB/s"))).toBe("10 MiB/s");
});
