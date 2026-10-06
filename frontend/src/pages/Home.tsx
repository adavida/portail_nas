import { useCallback, useEffect, useState } from "react";
import { listApps } from "../api/apps";
import type { App } from "../api/apps";
import { toast } from "../components/Toaster";
import "./Home.scss";

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
    <section>
      <h2>Applications</h2>
      {error ? (
        <p data-testid="apps-error">Erreur: {error}</p>
      ) : apps === null ? (
        <p data-testid="apps-loading">Chargement...</p>
      ) : apps.length === 0 ? (
        <p data-testid="apps-empty">Aucune application</p>
      ) : (
        <ul className="apps">
          {apps.map((app, i) => (
            <li key={`${app.name}-${app.url}`} data-testid={`app-${i}`}>
              <a
                className="app-link"
                data-testid={`app-link-${i}`}
                href={app.url}
                target="_blank"
                rel="noopener"
              >
                {app.icon ? (
                  <img src={app.icon} width={20} height={20} alt="" />
                ) : null}
                {app.name}
              </a>
              {app.description ? (
                <p className="app-desc">{app.description}</p>
              ) : null}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
