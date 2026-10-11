import "@testing-library/jest-dom";
import { afterEach } from "vitest";
import { toasts } from "../components/toast";

afterEach(() => {
  toasts.set([]);
});
