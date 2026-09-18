import { useCallback, useEffect, useState } from "react";
import { UsersTable, type User } from "../components/UsersTable";
import type { Group } from "../components/GroupsTable";
import { toast } from "../components/Toaster";
import { authHeader } from "../auth/oidc";

export default function Users() {
  const [users, setUsers] = useState<User[] | null>(null);
  const [groups, setGroups] = useState<Group[]>([]);
  const [error, setError] = useState<string | null>(null);

  const fetchUsers = useCallback(() => {
    const headers = authHeader();
    Promise.all([
      fetch("/api/users", { headers }).then(async (r) => {
        if (!r.ok) throw new Error(String(r.status));
        return (await r.json()) as User[];
      }),
      fetch("/api/groups", { headers }).then(async (r) => {
        if (!r.ok) throw new Error(String(r.status));
        return (await r.json()) as Group[];
      }),
    ])
      .then(([usersList, groupsList]) => {
        setUsers(
          usersList.map((u) => ({
            ...u,
            groups: groupsList
              .filter((g) => g.members.includes(u.uid))
              .map((g) => g.gid),
          })),
        );
        setGroups(groupsList);
      })
      .catch(() => {
        setError("error");
        toast("Erreur: chargement des utilisateurs impossible", true);
      });
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
        <UsersTable
          users={users}
          allGroups={groups.map((g) => g.gid)}
          onCreated={fetchUsers}
          onUpdated={fetchUsers}
          onDeleted={fetchUsers}
        />
      )}
    </section>
  );
}
