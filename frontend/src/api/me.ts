import { request } from "./http";

export type MeSaveResult = {
  profile_saved: boolean;
  attrs_saved: boolean;
  error?: string;
};

export function updateMe(
  name: string,
  email: string,
  attrs: { label: string; value: string }[],
): Promise<MeSaveResult> {
  return request("PUT", "/api/auth/me", { name, email, attrs });
}

export function updateMyPassword(password: string): Promise<unknown> {
  return request("PUT", "/api/auth/me/password", { password });
}

export type Policy = {
  minLength: number;
  defaultCountry: string;
};

export async function passwordPolicy(): Promise<Policy> {
  const cfg = await request<{
    password_min_length: number;
    default_phone_country?: string;
  }>("GET", "/api/auth/config");
  return {
    minLength: cfg.password_min_length,
    defaultCountry: cfg.default_phone_country ?? "FR",
  };
}
