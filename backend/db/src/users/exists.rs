use rusqlite::{Connection, params};

pub fn exists(conn: &Connection, uid: &str) -> Result<bool, String> {
    let count = conn
        .query_row(
            "SELECT count(*) FROM users WHERE uid = ?1",
            params![uid],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|e| format!("cannot look up user {uid}: {e}"))?;
    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::memory_db::memory_db;
    use crate::users::insert;

    #[test]
    fn exists_true_after_insert() {
        let conn = memory_db();
        insert(&conn, "alice", "Alice", "", "hash").unwrap();

        let found = exists(&conn, "alice").unwrap();

        assert!(found, "inserted uid should exist");
    }

    #[test]
    fn exists_false_for_unknown_uid() {
        let conn = memory_db();

        let found = exists(&conn, "ghost").unwrap();

        assert!(!found, "unknown uid should not exist");
    }
}
