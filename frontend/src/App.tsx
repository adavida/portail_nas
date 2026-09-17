import { useState } from "react";
import Groups from "./pages/Groups";
import Users from "./pages/Users";
import { Toaster } from "./components/Toaster";

export default function App() {
  const [tab, setTab] = useState<"users" | "groups">("users");

  return (
    <div>
      <nav style={{ display: "flex", gap: 8, marginBottom: 8 }}>
        <button data-testid="tab-users" onClick={() => setTab("users")}>
          Utilisateurs
        </button>
        <button data-testid="tab-groups" onClick={() => setTab("groups")}>
          Groupes
        </button>
      </nav>
      {tab === "users" ? <Users /> : <Groups />}
      <Toaster />
    </div>
  );
}
