import { useEffect, useState } from "react";
import { Route, Routes } from "react-router-dom";
import Groups from "./pages/Groups";
import Home from "./pages/Home";
import Users from "./pages/Users";
import Callback from "./pages/Callback";
import { Toaster } from "./components/Toaster";
import { whoami } from "./api/auth";
import { clearTokens, getToken, login, logout } from "./auth/oidc";
import "./App.scss";

type Theme = "light" | "dark";

function initialTheme(): Theme {
  const saved = localStorage.getItem("theme");
  if (saved === "light" || saved === "dark") return saved;
  return window.matchMedia?.("(prefers-color-scheme: dark)")?.matches
    ? "dark"
    : "light";
}

function Protected() {
  const [tab, setTab] = useState<"home" | "users" | "groups">("home");
  const [checked, setChecked] = useState(false);
  const [admin, setAdmin] = useState(false);
  const [theme, setTheme] = useState<Theme>(initialTheme);
  const token = getToken();

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem("theme", theme);
  }, [theme]);

  useEffect(() => {
    if (sessionStorage.getItem("login_in_progress") === "1") return;
    if (!token) {
      if (sessionStorage.getItem("logged_out") === "1") return;
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
        <div className="logged-out">
          <p data-testid="logged-out">Déconnecté</p>
          <button
            className="btn btn-primary"
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
    return (
      <p className="muted" data-testid="redirecting">
        Redirection vers Authelia...
      </p>
    );
  }

  if (!checked)
    return (
      <p className="muted" data-testid="checking">
        Vérification...
      </p>
    );

  return (
    <div className="layout">
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-badge">P</span>
          <span>Portail</span>
        </div>
        <nav className="nav">
          <button
            className={tab === "home" ? "nav-item active" : "nav-item"}
            data-testid="tab-home"
            onClick={() => setTab("home")}
          >
            Accueil
          </button>
          {admin && (
            <button
              className={tab === "users" ? "nav-item active" : "nav-item"}
              data-testid="tab-users"
              onClick={() => setTab("users")}
            >
              Utilisateurs
            </button>
          )}
          {admin && (
            <button
              className={tab === "groups" ? "nav-item active" : "nav-item"}
              data-testid="tab-groups"
              onClick={() => setTab("groups")}
            >
              Groupes
            </button>
          )}
        </nav>
        <div className="sidebar-spacer" />
        <button
          className="nav-item theme-toggle"
          role="switch"
          aria-checked={theme === "dark"}
          data-testid="theme-toggle"
          onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
        >
          <span>{theme === "dark" ? "Thème sombre" : "Thème clair"}</span>
          <span className="theme-toggle-track" aria-hidden="true">
            <span className="theme-toggle-thumb" />
          </span>
        </button>
        <button
          className="nav-item logout"
          data-testid="logout"
          onClick={logout}
        >
          Déconnexion
        </button>
      </aside>
      <main className="content">
        {tab === "home" ? <Home /> : tab === "users" ? <Users /> : <Groups />}
      </main>
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
