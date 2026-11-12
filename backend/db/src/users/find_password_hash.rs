use rusqlite::{Connection, OptionalExtension, params};

pub fn find_password_hash(conn: &Connection, uid: &str) -> Result<Option<String>, String> {
    conn.query_row(
        "SELECT password_hash FROM users WHERE uid = ?1",
        params![uid],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map_err(|e| format!("cannot read password hash for {uid}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::memory_db::memory_db;
    use crate::users::insert;

    #[test]
    fn find_password_hash_returns_stored_hash() {
        let conn = memory_db();
        insert(&conn, "alice", "Alice", "", "$argon2id$v=19$hash").unwrap();

        let hash = find_password_hash(&conn, "alice").unwrap();

        assert_eq!(
            hash.as_deref(),
            Some("$argon2id$v=19$hash"),
            "stored hash should roundtrip"
        );
    }

    #[test]
    fn find_password_hash_none_for_unknown_uid() {
        let conn = memory_db();

        let hash = find_password_hash(&conn, "ghost").unwrap();

        assert_eq!(hash, None, "unknown uid should have no hash");
    }
}
