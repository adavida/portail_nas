use rusqlite::{Connection, params};

pub fn set_password_hash(
    conn: &Connection,
    uid: &str,
    password_hash: &str,
) -> Result<usize, String> {
    conn.execute(
        "UPDATE users SET password_hash = ?2 WHERE uid = ?1",
        params![uid, password_hash],
    )
    .map_err(|e| format!("cannot update password hash for {uid}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::memory_db::memory_db;
    use crate::users::{find_password_hash, insert};

    #[test]
    fn set_password_hash_replaces_stored_hash() {
        let conn = memory_db();
        insert(&conn, "alice", "Alice", "", "old-hash").unwrap();

        let rows = set_password_hash(&conn, "alice", "new-hash").unwrap();

        assert_eq!(rows, 1, "one row should be updated");
        assert_eq!(
            find_password_hash(&conn, "alice").unwrap().as_deref(),
            Some("new-hash"),
            "hash should be replaced"
        );
    }

    #[test]
    fn set_password_hash_returns_zero_for_unknown_uid() {
        let conn = memory_db();

        let rows = set_password_hash(&conn, "ghost", "hash").unwrap();

        assert_eq!(rows, 0, "unknown uid must affect no row");
    }
}
