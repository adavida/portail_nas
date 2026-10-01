import { authHeader } from "../auth/oidc";

export async function whoami(): Promise<"ok" | "unauthorized" | "error"> {
  try {
    const r = await fetch("/api/auth/me", { headers: authHeader() });
    if (r.status === 401) return "unauthorized";
    return r.ok ? "ok" : "error";
  } catch {
    return "error";
  }
}
