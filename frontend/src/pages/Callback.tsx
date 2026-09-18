import { useEffect, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { handleCallback } from "../auth/oidc";

export default function Callback() {
  const [params] = useSearchParams();
  const navigate = useNavigate();
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const code = params.get("code");
    if (!code) {
      setError("code manquant");
      return;
    }
    handleCallback(code)
      .then(() => navigate("/", { replace: true }))
      .catch((e) => setError(String(e)));
  }, [params, navigate]);

  if (error) return <p data-testid="callback-error">Erreur: {error}</p>;
  return <p data-testid="callback-loading">Connexion...</p>;
}
