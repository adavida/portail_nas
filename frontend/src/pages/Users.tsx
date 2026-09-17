import { useEffect, useState } from "react";
import { UsersTable, type User } from "../components/UsersTable";

export default function Users() {
  const [users, setUsers] = useState<User[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetch("/api/users")
      .then((r) => {
        if (!r.ok) throw new Error(String(r.status));
        return r.json();
      })
      .then((d) => setUsers(d as User[]))
      .catch(() => setError("error"));
  }, []);

  if (error) return <p data-testid="users-error">Erreur: {error}</p>;
  if (users === null) return <p data-testid="users-loading">Chargement...</p>;

  return (
    <section style={{ marginTop: 24 }}>
      <h2>Utilisateurs</h2>
      <UsersTable users={users} />
    </section>
  );
}
