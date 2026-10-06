/* eslint-disable react-refresh/only-export-components -- toast() and Toaster() coupled by design */
import { useEffect, useState } from "react";
import "./Toaster.scss";

export type Toast = { id: number; message: string; isError: boolean };

let counter = 0;
let listeners: ((t: Toast) => void)[] = [];

export function toast(message: string, isError = false) {
  counter += 1;
  const t = { id: counter, message, isError };
  listeners.forEach((l) => l(t));
}

const DURATION_MS = 15000;

export function Toaster() {
  const [toasts, setToasts] = useState<Toast[]>([]);

  useEffect(() => {
    const listener = (t: Toast) => {
      setToasts((prev) => [...prev, t]);
      setTimeout(() => {
        setToasts((prev) => prev.filter((x) => x.id !== t.id));
      }, DURATION_MS);
    };
    listeners.push(listener);
    return () => {
      listeners = listeners.filter((l) => l !== listener);
    };
  }, []);

  if (!toasts.length) return null;

  return (
    <div data-testid="toaster" className="toaster">
      {toasts.map((t) => (
        <div
          key={t.id}
          className={t.isError ? "toast toast--error" : "toast"}
          data-testid={t.isError ? "toast-error" : "toast"}
        >
          {t.message}
        </div>
      ))}
    </div>
  );
}
