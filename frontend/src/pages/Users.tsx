import { useCallback, useEffect, useState } from "react";
import { UsersTable, type User } from "../components/UsersTable";

export default function Users() {
  const [users, setUsers] = useState<User[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  const fetchUsers = useCallback(() => {
    fetch("/api/users")
      .then((r) => {
        if (!r.ok) throw new Error(String(r.status));
        return r.json();
      })
      .then((d) => setUsers(d as User[]))
      .catch(() => setError("error"));
  }, []);

  useEffect(() => {
    fetchUsers();
  }, [fetchUsers]);

  return (
    <section style={{ marginTop: 24 }}>
      <h2>Utilisateurs</h2>
      {error ? (
        <p data-testid="users-error">Erreur: {error}</p>
      ) : users === null ? (
        <p data-testid="users-loading">Chargement...</p>
      ) : (
        <UsersTable users={users} onCreated={fetchUsers} />
      )}
    </section>
  );
}
