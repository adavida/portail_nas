use rusqlite::{Connection, params};

pub fn exists(conn: &Connection, gid: &str) -> Result<bool, String> {
    let count = conn
        .query_row(
            "SELECT count(*) FROM groups WHERE gid = ?1",
            params![gid],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|e| format!("cannot look up group {gid}: {e}"))?;
    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::groups::insert;
    use crate::test_helpers::memory_db::memory_db;

    #[test]
    fn exists_true_after_insert() {
        let conn = memory_db();
        insert(&conn, "devs", "Devs", "").unwrap();

        let found = exists(&conn, "devs").unwrap();

        assert!(found, "inserted gid should exist");
    }

    #[test]
    fn exists_false_for_unknown_gid() {
        let conn = memory_db();

        let found = exists(&conn, "ghost").unwrap();

        assert!(!found, "unknown gid should not exist");
    }
}
