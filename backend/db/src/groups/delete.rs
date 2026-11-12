use rusqlite::{Connection, params};

pub fn delete(conn: &Connection, gid: &str) -> Result<usize, String> {
    conn.execute("DELETE FROM groups WHERE gid = ?1", params![gid])
        .map_err(|e| format!("cannot delete group {gid}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::groups::{add_member, exists, find_memberships, insert};
    use crate::test_helpers::memory_db::memory_db;
    use crate::users;

    #[test]
    fn delete_removes_group_and_cascades_memberships() {
        let conn = memory_db();
        users::insert(&conn, "alice", "Alice", "", "hash").unwrap();
        insert(&conn, "devs", "Devs", "").unwrap();
        add_member(&conn, "devs", "alice").unwrap();

        let rows = delete(&conn, "devs").unwrap();

        assert_eq!(rows, 1, "one group row should be deleted");
        assert!(!exists(&conn, "devs").unwrap(), "group should be gone");
        assert!(
            find_memberships(&conn).unwrap().is_empty(),
            "memberships of the deleted group should cascade away"
        );
        assert!(
            users::exists(&conn, "alice").unwrap(),
            "cascade must not delete the users themselves"
        );
    }

    #[test]
    fn delete_returns_zero_for_unknown_gid() {
        let conn = memory_db();

        let rows = delete(&conn, "ghost").unwrap();

        assert_eq!(rows, 0, "unknown gid must affect no row");
    }
}
