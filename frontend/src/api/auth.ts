import { authHeader } from "../auth/oidc";

export type AttrEntry = {
  label: string;
  value: string;
  type: "text" | "number" | "email" | "telephone";
  max_chars: number | null;
  min: number | null;
  max: number | null;
  names?: Record<string, string>;
};

export type Me = {
  sub: string;
  preferred_username: string | null;
  name: string | null;
  email: string | null;
  groups: string[];
  is_admin: boolean;
  attrs?: AttrEntry[];
};

const anonymous: Me = {
  sub: "",
  preferred_username: null,
  name: null,
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
