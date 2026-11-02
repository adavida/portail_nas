import { authHeader } from "../auth/oidc";

export type Me = {
  sub: string;
  preferred_username: string | null;
  email: string | null;
  groups: string[];
  is_admin: boolean;
};

const anonymous: Me = {
  sub: "",
  preferred_username: null,
  email: null,
  groups: [],
  is_admin: false,
};

export async function whoami(): Promise<Me | "relogin"> {
  try {
    const r = await fetch("/api/auth/me", { headers: authHeader() });
    if (r.status === 401) return "relogin";
    if (!r.ok) return anonymous;
    return (await r.json()) as Me;
  } catch {
    return anonymous;
  }
}
