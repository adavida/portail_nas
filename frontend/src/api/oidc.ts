// Provider's password login: POST /api/oidc/login (relatif, proxy vite).
// Returns the redirect URI on success, or the error + status so the page can
// translate the 401 credentials case itself (rule: no silent failure).

export type LoginResponse =
  { redirect: string } | { error: string; status: number };

export async function loginWithPassword(
  req: string,
  username: string,
  password: string,
): Promise<LoginResponse> {
  const res = await fetch("/api/oidc/login", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ req, username, password }),
  });
  const j = (await res.json().catch(() => ({}))) as {
    redirect?: string;
    error?: string;
  };
  if (res.ok && j.redirect) return { redirect: j.redirect };
  return { error: j.error || `error ${res.status}`, status: res.status };
}
