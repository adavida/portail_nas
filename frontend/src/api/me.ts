import { request } from "./http";

export function updateMe(name: string, email: string): Promise<unknown> {
  return request("PUT", "/api/auth/me", { name, email });
}

export function updateMyPassword(password: string): Promise<unknown> {
  return request("PUT", "/api/auth/me/password", { password });
}

export async function passwordPolicy(): Promise<number> {
  const cfg = await request<{ password_min_length: number }>(
    "GET",
    "/api/auth/config",
  );
  return cfg.password_min_length;
}
