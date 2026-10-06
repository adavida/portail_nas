import { useCallback, useEffect, useState } from "react";
import { listGroups } from "../api/groups";
import type { Group } from "../api/groups";
import { GroupsTable } from "../components/GroupsTable";
import { toast } from "../components/Toaster";

export default function Groups() {
  const [groups, setGroups] = useState<Group[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  const fetchGroups = useCallback(() => {
    listGroups()
      .then((d) => setGroups(d))
      .catch(() => {
        setError("error");
        toast("Erreur: chargement des groupes impossible", true);
      });
  }, []);

  useEffect(() => {
    fetchGroups();
  }, [fetchGroups]);

  return (
    <section>
      <h2>Groupes</h2>
      {error ? (
        <p data-testid="groups-error">Erreur: {error}</p>
      ) : groups === null ? (
        <p data-testid="groups-loading">Chargement...</p>
      ) : (
        <div className="card">
          <GroupsTable
            groups={groups}
            onUpdated={fetchGroups}
            onDeleted={fetchGroups}
          />
        </div>
      )}
    </section>
  );
}
