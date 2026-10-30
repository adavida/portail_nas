import { request } from "./http";

export type DebugClient = {
  id: string;
  enabled: boolean;
  has_secret: boolean;
  redirect_uris: string[];
  scopes: string[];
  required_group: string;
  forbidden_group: string;
};

export type DebugInfo = {
  issuer: string;
  client_id: string;
  redirect_uri: string;
  key_bits: number;
  pending_ttl_secs: number;
  code_ttl_secs: number;
  session_ttl_secs: number;
  access_ttl_secs: number;
  clients: DebugClient[];
};

export function getDebug() {
  return request<DebugInfo>("GET", "/api/auth/debug");
}

export function getClaims(uid: string, clientId: string) {
  return request<Record<string, unknown>>(
    "GET",
    `/api/auth/debug/claims?uid=${encodeURIComponent(uid)}&client_id=${encodeURIComponent(clientId)}`,
  );
}
