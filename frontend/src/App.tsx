import { useEffect, useState } from "react";
import { Route, Routes } from "react-router-dom";
import Groups from "./pages/Groups";
import Users from "./pages/Users";
import Callback from "./pages/Callback";
import { Toaster } from "./components/Toaster";
import { authHeader, clearTokens, getToken, login, logout } from "./auth/oidc";

function Protected() {
  const [tab, setTab] = useState<"users" | "groups">("users");
  const [checked, setChecked] = useState(false);
  const [admin, setAdmin] = useState(false);
  const token = getToken();

  useEffect(() => {
    if (sessionStorage.getItem("login_in_progress") === "1") return;
    if (!token) {
      sessionStorage.setItem("login_in_progress", "1");
      login();
      return;
    }
    fetch("/api/auth/me", { headers: authHeader() })
      .then((r) => {
        if (r.status === 401) {
          clearTokens();
          sessionStorage.setItem("login_in_progress", "1");
          login();
          return;
        }
        setAdmin(r.ok);
        setChecked(true);
        sessionStorage.removeItem("login_in_progress");
      })
      .catch(() => {
        setChecked(true);
        sessionStorage.removeItem("login_in_progress");
      });
  }, [token]);

  if (!token) {
    if (sessionStorage.getItem("logged_out") === "1")
      return (
        <div>
          <p data-testid="logged-out">Déconnecté</p>
          <button
            data-testid="login-button"
            onClick={() => {
              sessionStorage.removeItem("logged_out");
              sessionStorage.setItem("login_in_progress", "1");
              login();
            }}
          >
            Se connecter
          </button>
        </div>
      );
    return <p data-testid="redirecting">Redirection vers Authelia...</p>;
  }

  if (!checked) return <p data-testid="checking">Vérification...</p>;

  if (!admin)
    return (
      <div>
        <p data-testid="forbidden">
          Accès réservé aux administrateurs — connectez-vous avec{" "}
          <code>admin</code>
        </p>
        <button data-testid="logout" onClick={logout}>
          Déconnexion
        </button>
      </div>
    );

  return (
    <div>
      <nav style={{ display: "flex", gap: 8, marginBottom: 8 }}>
        <button data-testid="tab-users" onClick={() => setTab("users")}>
          Utilisateurs
        </button>
        <button data-testid="tab-groups" onClick={() => setTab("groups")}>
          Groupes
        </button>
        <button data-testid="logout" onClick={logout}>
          Déconnexion
        </button>
      </nav>
      {tab === "users" ? <Users /> : <Groups />}
      <Toaster />
    </div>
  );
}

export default function App() {
  return (
    <Routes>
      <Route path="/callback" element={<Callback />} />
      <Route path="/*" element={<Protected />} />
    </Routes>
  );
}
