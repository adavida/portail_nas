import { useState } from "react";

export function CreateUserForm({ onCreated }: { onCreated: () => void }) {
  const [uid, setUid] = useState("");
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!uid.trim() || !name.trim() || !password) {
      setError("uid, nom et mot de passe requis");
      return;
    }
    setError(null);
    const res = await fetch("/api/users", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ uid, name, email, password }),
    });
    if (!res.ok) {
      const j = await res.json().catch(() => ({}));
      setError(j.error || `error ${res.status}`);
      return;
    }
    setUid("");
    setName("");
    setEmail("");
    setPassword("");
    onCreated();
  };

  return (
    <form
      data-testid="create-form"
      onSubmit={submit}
      style={{ display: "flex", gap: 8, marginBottom: 16, flexWrap: "wrap" }}
    >
      <input
        data-testid="input-uid"
        placeholder="uid"
        value={uid}
        onChange={(e) => setUid(e.target.value)}
        required
      />
      <input
        data-testid="input-name"
        placeholder="nom"
        value={name}
        onChange={(e) => setName(e.target.value)}
        required
      />
      <input
        data-testid="input-email"
        placeholder="email"
        value={email}
        onChange={(e) => setEmail(e.target.value)}
      />
      <input
        data-testid="input-password"
        placeholder="mot de passe"
        type="password"
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        required
      />
      <button data-testid="create-button" type="submit">
        Créer
      </button>
      {error && <span data-testid="create-error">{error}</span>}
    </form>
  );
}
