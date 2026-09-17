import { useEffect, useState } from "react";
import { HealthBadge } from "../components/HealthBadge";

export default function Home() {
  const [status, setStatus] = useState<string>("loading...");

  useEffect(() => {
    fetch("/api/health")
      .then((r) => r.json())
      .then((d) => setStatus(d.status))
      .catch(() => setStatus("error"));
  }, []);

  return (
    <main style={{ fontFamily: "sans-serif", padding: 24 }}>
      <h1>Portail LDAP</h1>
      <HealthBadge status={status} />
    </main>
  );
}
