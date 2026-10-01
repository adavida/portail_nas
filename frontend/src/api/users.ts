import { request } from "./http";

export type User = {
  uid: string;
  name: string;
  email: string;
  groups?: string[];
};

export function listUsers() {
  return request<User[]>("GET", "/api/users");
}

export function createUser(input: {
  uid: string;
  name: string;
  email: string;
  password: string;
}) {
  return request<void>("POST", "/api/users", input);
}

export function updateUser(
  uid: string,
  input: { name: string; email: string },
) {
  return request<void>("PUT", `/api/users/${encodeURIComponent(uid)}`, input);
}

export function updatePassword(uid: string, password: string) {
  return request<void>(
    "PUT",
    `/api/users/${encodeURIComponent(uid)}/password`,
    { password },
  );
}

export function deleteUser(uid: string) {
  return request<void>("DELETE", `/api/users/${encodeURIComponent(uid)}`);
}
