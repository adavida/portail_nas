import { request } from "./http";

export type App = {
  name: string;
  url: string;
  description?: string;
  icon?: string | null;
  required_group?: string;
  forbidden_group?: string;
};

export function listApps() {
  return request<App[]>("GET", "/apps.json");
}
