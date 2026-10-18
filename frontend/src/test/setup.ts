import "@testing-library/jest-dom/vitest";
import { afterEach } from "vitest";
import { toasts } from "../components/toast";
import { mockMedia } from "./mock_media";

mockMedia(false);

afterEach(() => {
  toasts.set([]);
  mockMedia(false);
});
