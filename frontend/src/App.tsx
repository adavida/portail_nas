import { useEffect, useState } from "react";
import { Route, Routes } from "react-router-dom";
import Groups from "./pages/Groups";
import Home from "./pages/Home";
import Users from "./pages/Users";
import Callback from "./pages/Callback";
import { Toaster } from "./components/Toaster";
import { whoami } from "./api/auth";
import { clearTokens, getToken, login, logout } from "./auth/oidc";

function Protected() {
  const [tab, setTab] = useState<"home" | "users" | "groups">("home");
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
    whoami().then((s) => {
      if (s === "relogin") {
        clearTokens();
        sessionStorage.setItem("login_in_progress", "1");
        login();
        return;
      }
      setAdmin(s === "admin");
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

  return (
    <div>
      <nav style={{ display: "flex", gap: 8, marginBottom: 8 }}>
        <button data-testid="tab-home" onClick={() => setTab("home")}>
          Accueil
        </button>
        {admin && (
          <button data-testid="tab-users" onClick={() => setTab("users")}>
            Utilisateurs
          </button>
        )}
        {admin && (
          <button data-testid="tab-groups" onClick={() => setTab("groups")}>
            Groupes
          </button>
        )}
        <button data-testid="logout" onClick={logout}>
          Déconnexion
        </button>
      </nav>
      {tab === "home" ? <Home /> : tab === "users" ? <Users /> : <Groups />}
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
