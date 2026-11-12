use rusqlite::{Connection, params};

pub fn delete(conn: &Connection, uid: &str) -> Result<usize, String> {
    conn.execute("DELETE FROM users WHERE uid = ?1", params![uid])
        .map_err(|e| format!("cannot delete user {uid}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::memory_db::memory_db;
    use crate::users::{exists, find_all, insert};

    #[test]
    fn delete_removes_user_and_memberships() {
        let conn = memory_db();
        insert(&conn, "alice", "Alice", "", "hash").unwrap();
        crate::groups::insert(&conn, "devs", "Devs", "").unwrap();
        crate::groups::add_member(&conn, "devs", "alice").unwrap();

        let rows = delete(&conn, "alice").unwrap();

        assert_eq!(rows, 1, "one user row should be deleted");
        assert!(!exists(&conn, "alice").unwrap(), "user should be gone");
        assert!(
            crate::groups::find_memberships(&conn)
                .unwrap()
                .iter()
                .all(|(_, uid)| uid != "alice"),
            "memberships of the deleted user should be gone"
        );
        assert!(find_all(&conn).unwrap().is_empty(), "no user left");
    }

    #[test]
    fn delete_returns_zero_for_unknown_uid() {
        let conn = memory_db();

        let rows = delete(&conn, "ghost").unwrap();

        assert_eq!(rows, 0, "unknown uid must affect no row");
    }
}
