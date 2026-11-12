use rusqlite::{Connection, params};

pub fn insert(
    conn: &Connection,
    uid: &str,
    name: &str,
    email: &str,
    password_hash: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO users (uid, name, email, password_hash) VALUES (?1, ?2, ?3, ?4)",
        params![uid, name, email, password_hash],
    )
    .map_err(|e| format!("cannot insert user {uid}: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::memory_db::memory_db;
    use crate::users::{exists, find_all};

    #[test]
    fn insert_persists_full_record() {
        let conn = memory_db();

        insert(&conn, "alice", "Alice A", "a@ex.com", "hash").unwrap();

        assert!(exists(&conn, "alice").unwrap(), "uid should exist");
        assert_eq!(
            find_all(&conn).unwrap()[0].email,
            "a@ex.com",
            "email should be persisted"
        );
    }

    #[test]
    fn insert_rejects_duplicate_uid() {
        let conn = memory_db();
        insert(&conn, "alice", "First", "", "hash").unwrap();

        let err = insert(&conn, "alice", "Second", "", "hash").unwrap_err();

        assert!(
            err.contains("cannot insert user alice"),
            "duplicate primary key must fail loudly: {err}"
        );
    }
}
