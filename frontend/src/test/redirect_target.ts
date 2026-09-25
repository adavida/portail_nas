import { login } from "../auth/oidc";

export async function redirectTarget(): Promise<URL> {
  let href = "";
  Object.defineProperty(window, "location", {
    configurable: true,
    value: {
      get href() {
        return href;
      },
      set href(v: string) {
        href = v;
      },
    },
  });
  await login();
  return new URL(href);
}
