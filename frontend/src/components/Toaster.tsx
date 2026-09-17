/* eslint-disable react-refresh/only-export-components -- toast() et Toaster() couplés par design */
import { useEffect, useState } from "react";

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
    <div
      data-testid="toaster"
      style={{ position: "fixed", bottom: 16, left: 16, zIndex: 1000 }}
    >
      {toasts.map((t) => (
        <div
          key={t.id}
          data-testid={t.isError ? "toast-error" : "toast"}
          style={{
            background: t.isError ? "#fff0f0" : "#f0fff0",
            border: `1px solid ${t.isError ? "#cc3333" : "#33aa33"}`,
            color: t.isError ? "#991111" : "#117711",
            padding: "8px 12px",
            marginTop: 8,
            borderRadius: 6,
            boxShadow: "0 2px 6px rgba(0,0,0,.15)",
          }}
        >
          {t.message}
        </div>
      ))}
    </div>
  );
}
