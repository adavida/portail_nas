import { authHeader } from "../auth/oidc";

export async function whoami(): Promise<"admin" | "user" | "relogin"> {
  try {
    const r = await fetch("/api/auth/me", { headers: authHeader() });
    if (r.status === 401) return "relogin";
    return r.ok ? "admin" : "user";
  } catch {
    return "user";
  }
}
