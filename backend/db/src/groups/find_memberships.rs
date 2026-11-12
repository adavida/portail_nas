use rusqlite::Connection;

pub fn find_memberships(conn: &Connection) -> Result<Vec<(String, String)>, String> {
    let mut statement = conn
        .prepare("SELECT gid, uid FROM group_members ORDER BY gid ASC, uid ASC")
        .map_err(|e| format!("cannot prepare membership list: {e}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| format!("cannot list memberships: {e}"))?;
    rows.collect::<Result<Vec<(String, String)>, _>>()
        .map_err(|e| format!("cannot read membership rows: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::groups::{add_member, insert};
    use crate::test_helpers::memory_db::memory_db;
    use crate::users;

    #[test]
    fn find_memberships_returns_pairs_sorted() {
        let conn = memory_db();
        users::insert(&conn, "alice", "Alice", "", "hash").unwrap();
        users::insert(&conn, "bob", "Bob", "", "hash").unwrap();
        insert(&conn, "devs", "Devs", "").unwrap();
        add_member(&conn, "devs", "bob").unwrap();
        add_member(&conn, "devs", "alice").unwrap();

        let memberships = find_memberships(&conn).unwrap();

        assert_eq!(
            memberships,
            vec![
                ("devs".to_string(), "alice".to_string()),
                ("devs".to_string(), "bob".to_string()),
            ],
            "pairs should be sorted by gid then uid"
        );
    }
}
