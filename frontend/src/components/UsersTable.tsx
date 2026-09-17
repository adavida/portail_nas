export type User = { uid: string; name: string; email: string };

export function UsersTable({ users }: { users: User[] }) {
  if (users.length === 0)
    return <p data-testid="users-empty">Aucun utilisateur</p>;
  return (
    <table
      data-testid="users-table"
      style={{ borderCollapse: "collapse", width: "100%" }}
    >
      <thead>
        <tr>
          <th
            style={{ border: "1px solid #ccc", padding: 8, textAlign: "left" }}
          >
            UID
          </th>
          <th
            style={{ border: "1px solid #ccc", padding: 8, textAlign: "left" }}
          >
            Nom
          </th>
          <th
            style={{ border: "1px solid #ccc", padding: 8, textAlign: "left" }}
          >
            Email
          </th>
        </tr>
      </thead>
      <tbody>
        {users.map((u) => (
          <tr key={u.uid} data-testid={`user-row-${u.uid}`}>
            <td style={{ border: "1px solid #ccc", padding: 8 }}>{u.uid}</td>
            <td style={{ border: "1px solid #ccc", padding: 8 }}>{u.name}</td>
            <td style={{ border: "1px solid #ccc", padding: 8 }}>{u.email}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
