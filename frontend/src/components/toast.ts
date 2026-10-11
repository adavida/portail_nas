import { writable } from "svelte/store";

export type Toast = { id: number; message: string; isError: boolean };

export const toasts = writable<Toast[]>([]);

let counter = 0;

const DURATION_MS = 15000;

export function toast(message: string, isError = false) {
  counter += 1;
  const t = { id: counter, message, isError };
  toasts.update((prev) => [...prev, t]);
  setTimeout(
    () => toasts.update((prev) => prev.filter((x) => x.id !== t.id)),
    DURATION_MS,
  );
}
