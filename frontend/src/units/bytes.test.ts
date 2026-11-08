import { expect, test } from "vitest";
import { Bytes } from "./bytes";

test("bytes scales labeled units to base bytes", () => {
  expect(new Bytes("900 B").value).toBe(900);
  expect(new Bytes("2.5 MiB").value).toBe(2.5 * 1024 ** 2);
  expect(new Bytes("16 GiB").value).toBe(16 * 1024 ** 3);
});

test("bytes keeps the wire text for display", () => {
  expect(String(new Bytes("2.5 MiB"))).toBe("2.5 MiB");
});
