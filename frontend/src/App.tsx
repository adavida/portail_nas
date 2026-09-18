import { useEffect, useState } from "react";
import { Route, Routes, useNavigate } from "react-router-dom";
import Groups from "./pages/Groups";
import Users from "./pages/Users";
import Callback from "./pages/Callback";
import { Toaster } from "./components/Toaster";
import { clearTokens, getToken, login } from "./auth/oidc";

function Protected() {
  const [tab, setTab] = useState<"users" | "groups">("users");
  const navigate = useNavigate();
  const token = getToken();

  useEffect(() => {
    if (!token) {
      login();
    }
  }, [token]);

  if (!token)
    return <p data-testid="redirecting">Redirection vers Authelia...</p>;

  return (
    <div>
      <nav style={{ display: "flex", gap: 8, marginBottom: 8 }}>
        <button data-testid="tab-users" onClick={() => setTab("users")}>
          Utilisateurs
        </button>
        <button data-testid="tab-groups" onClick={() => setTab("groups")}>
          Groupes
        </button>
        <button
          data-testid="logout"
          onClick={() => {
            clearTokens();
            navigate("/");
          }}
        >
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
