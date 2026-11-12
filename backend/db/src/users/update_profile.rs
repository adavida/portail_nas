use rusqlite::{Connection, params};

pub fn update_profile(
    conn: &Connection,
    uid: &str,
    name: &str,
    email: &str,
) -> Result<usize, String> {
    conn.execute(
        "UPDATE users SET name = ?2, email = ?3 WHERE uid = ?1",
        params![uid, name, email],
    )
    .map_err(|e| format!("cannot update user {uid}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::memory_db::memory_db;
    use crate::users::{find_all, insert};

    #[test]
    fn update_profile_changes_name_and_email() {
        let conn = memory_db();
        insert(&conn, "alice", "Old", "old@ex.com", "hash").unwrap();

        let rows = update_profile(&conn, "alice", "New Name", "new@ex.com").unwrap();

        assert_eq!(rows, 1, "one row should be updated");
        let users = find_all(&conn).unwrap();
        assert_eq!(users[0].name, "New Name", "name should be updated");
        assert_eq!(users[0].email, "new@ex.com", "email should be updated");
    }

    #[test]
    fn update_profile_returns_zero_for_unknown_uid() {
        let conn = memory_db();

        let rows = update_profile(&conn, "ghost", "New", "").unwrap();

        assert_eq!(rows, 0, "unknown uid must affect no row");
    }
}
