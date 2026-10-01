import { useCallback, useEffect, useState } from "react";
import { listGroups } from "../api/groups";
import type { Group } from "../api/groups";
import { listUsers } from "../api/users";
import type { User } from "../api/users";
import { UsersTable } from "../components/UsersTable";
import { toast } from "../components/Toaster";

export default function Users() {
  const [users, setUsers] = useState<User[] | null>(null);
  const [groups, setGroups] = useState<Group[]>([]);
  const [error, setError] = useState<string | null>(null);

  const fetchUsers = useCallback(() => {
    listGroups()
      .then(setGroups)
      .catch(() => toast("Erreur: chargement des groupes impossible", true));
    listUsers()
      .then(setUsers)
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
