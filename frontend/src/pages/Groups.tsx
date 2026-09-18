import { useCallback, useEffect, useState } from "react";
import { GroupsTable, type Group } from "../components/GroupsTable";
import { toast } from "../components/Toaster";
import { authHeader } from "../auth/oidc";

export default function Groups() {
  const [groups, setGroups] = useState<Group[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  const fetchGroups = useCallback(() => {
    fetch("/api/groups", { headers: authHeader() })
      .then((r) => {
        if (!r.ok) throw new Error(String(r.status));
        return r.json();
      })
      .then((d) => setGroups(d as Group[]))
      .catch(() => {
        setError("error");
        toast("Erreur: chargement des groupes impossible", true);
      });
  }, []);

  useEffect(() => {
    fetchGroups();
  }, [fetchGroups]);

  return (
    <section style={{ marginTop: 24 }}>
      <h2>Groupes</h2>
      {error ? (
        <p data-testid="groups-error">Erreur: {error}</p>
      ) : groups === null ? (
        <p data-testid="groups-loading">Chargement...</p>
      ) : (
        <GroupsTable
          groups={groups}
          onUpdated={fetchGroups}
          onDeleted={fetchGroups}
        />
      )}
    </section>
  );
}
