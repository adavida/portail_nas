import { useCallback, useEffect, useState } from "react";
import { listApps } from "../api/apps";
import type { App } from "../api/apps";
import { toast } from "../components/Toaster";

export default function Home() {
  const [apps, setApps] = useState<App[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  const fetchApps = useCallback(() => {
    listApps()
      .then(setApps)
      .catch(() => {
        setError("error");
        toast("Erreur: chargement des applications impossible", true);
      });
  }, []);

  useEffect(() => {
    fetchApps();
  }, [fetchApps]);

  return (
    <section style={{ marginTop: 24 }}>
      <h2>Applications</h2>
      {error ? (
        <p data-testid="apps-error">Erreur: {error}</p>
      ) : apps === null ? (
        <p data-testid="apps-loading">Chargement...</p>
      ) : apps.length === 0 ? (
        <p data-testid="apps-empty">Aucune application</p>
      ) : (
        <ul
          style={{
            listStyle: "none",
            padding: 0,
            display: "flex",
            flexDirection: "column",
            gap: 12,
          }}
        >
          {apps.map((app, i) => (
            <li key={`${app.name}-${app.url}`} data-testid={`app-${i}`}>
              <a
                data-testid={`app-link-${i}`}
                href={app.url}
                target="_blank"
                rel="noopener"
                style={{ fontWeight: 600 }}
              >
                {app.icon ? (
                  <img
                    src={app.icon}
                    width={20}
                    height={20}
                    alt=""
                    style={{ marginRight: 8, verticalAlign: "middle" }}
                  />
                ) : null}
                {app.name}
              </a>
              {app.description ? (
                <p style={{ margin: "4px 0 0", color: "#555" }}>
                  {app.description}
                </p>
              ) : null}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
