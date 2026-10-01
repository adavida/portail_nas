import { request } from "./http";

export type Group = {
  gid: string;
  name: string;
  description: string;
};

export function listGroups() {
  return request<Group[]>("GET", "/api/groups");
}

export function createGroup(input: {
  gid: string;
  name: string;
  description: string;
  members: string[];
}) {
  return request<void>("POST", "/api/groups", input);
}

export function updateGroup(
  gid: string,
  input: { name: string; description: string },
) {
  return request<void>("PUT", `/api/groups/${encodeURIComponent(gid)}`, input);
}

export function deleteGroup(gid: string) {
  return request<void>("DELETE", `/api/groups/${encodeURIComponent(gid)}`);
}

export function addMember(gid: string, uid: string) {
  return request<void>(
    "POST",
    `/api/groups/${encodeURIComponent(gid)}/members`,
    {
      uid,
    },
  );
}

export function removeMember(gid: string, uid: string) {
  return request<void>(
    "DELETE",
    `/api/groups/${encodeURIComponent(gid)}/members/${encodeURIComponent(uid)}`,
  );
}
