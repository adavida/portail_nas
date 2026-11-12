use rusqlite::{Connection, params};

pub fn insert(conn: &Connection, gid: &str, name: &str, description: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO groups (gid, name, description) VALUES (?1, ?2, ?3)",
        params![gid, name, description],
    )
    .map_err(|e| format!("cannot insert group {gid}: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::groups::{exists, find_all};
    use crate::test_helpers::memory_db::memory_db;

    #[test]
    fn insert_persists_group() {
        let conn = memory_db();

        insert(&conn, "devs", "Devs", "dev team").unwrap();

        assert!(exists(&conn, "devs").unwrap(), "gid should exist");
        assert_eq!(
            find_all(&conn).unwrap()[0].description,
            "dev team",
            "description should be persisted"
        );
    }

    #[test]
    fn insert_rejects_duplicate_gid() {
        let conn = memory_db();
        insert(&conn, "devs", "First", "").unwrap();

        let err = insert(&conn, "devs", "Second", "").unwrap_err();

        assert!(
            err.contains("cannot insert group devs"),
            "duplicate primary key must fail loudly: {err}"
        );
    }
}
