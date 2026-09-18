import { useEffect, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { handleCallback } from "../auth/oidc";

export default function Callback() {
  const [params] = useSearchParams();
  const navigate = useNavigate();
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const code = params.get("code");
    const state = params.get("state");
    const storedState = sessionStorage.getItem("oidc_state");
    if (state && storedState && state !== storedState) {
      setError(`state mismatch: ${state} vs ${storedState}`);
      return;
    }
    if (!code) {
      const err = params.get("error");
      const desc = params.get("error_description");
      setError(err ? `${err}: ${desc ?? ""}` : "code missing");
      return;
    }
    // StrictMode double-mount guard + code déjà échangé
    const key = `handled_${code}`;
    if (sessionStorage.getItem(key)) return;
    sessionStorage.setItem(key, "1");
    // si on a déjà un token, ne pas rééchanger
    if (localStorage.getItem("access_token")) {
      sessionStorage.removeItem("login_in_progress");
      navigate("/", { replace: true });
      return;
    }
    handleCallback(code)
      .then(() => {
        sessionStorage.removeItem("login_in_progress");
        sessionStorage.removeItem("logged_out");
        sessionStorage.removeItem("oidc_state");
        navigate("/", { replace: true });
      })
      .catch((e) => {
        sessionStorage.removeItem("login_in_progress");
        setError(String(e));
      });
  }, [params, navigate]);

  if (error) return <p data-testid="callback-error">Erreur: {error}</p>;
  return <p data-testid="callback-loading">Connexion...</p>;
}
