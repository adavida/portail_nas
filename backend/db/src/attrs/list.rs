use rusqlite::{Connection, params};

pub fn list_attrs(conn: &Connection, uid: &str) -> Result<Vec<(String, String)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT l.name, a.value FROM user_attrs a
             JOIN attr_labels l ON l.id = a.label_id
             WHERE a.user_id = ?1
             ORDER BY l.name",
        )
        .map_err(|e| format!("cannot prepare attr list: {e}"))?;
    let rows = stmt
        .query_map(params![uid], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| format!("cannot list attrs of user {uid}: {e}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("cannot read attrs of user {uid}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attrs::set_attr;
    use crate::test_helpers::memory_db::memory_db;
    use crate::users::insert;

    #[test]
    fn list_attrs_returns_sorted_pairs() {
        let conn = memory_db();
        insert(&conn, "alice", "A", "", "h").unwrap();
        set_attr(&conn, "alice", "zzz", "2").unwrap();
        set_attr(&conn, "alice", "aaa", "1").unwrap();

        let attrs = list_attrs(&conn, "alice").unwrap();

        assert_eq!(
            attrs,
            vec![
                ("aaa".to_string(), "1".to_string()),
                ("zzz".to_string(), "2".to_string())
            ],
            "pairs must come back sorted by label"
        );
    }
}
