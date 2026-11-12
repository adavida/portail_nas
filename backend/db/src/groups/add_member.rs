use rusqlite::{Connection, params};

pub fn add_member(conn: &Connection, gid: &str, uid: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO group_members (gid, uid) VALUES (?1, ?2)",
        params![gid, uid],
    )
    .map_err(|e| format!("cannot add {uid} to group {gid}: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::groups::{insert, members_of};
    use crate::test_helpers::memory_db::memory_db;
    use crate::users;

    #[test]
    fn add_member_appends_uid_to_group() {
        let conn = memory_db();
        users::insert(&conn, "alice", "Alice", "", "hash").unwrap();
        insert(&conn, "devs", "Devs", "").unwrap();

        add_member(&conn, "devs", "alice").unwrap();

        assert_eq!(
            members_of(&conn, "devs").unwrap(),
            vec!["alice".to_string()],
            "member should be listed"
        );
    }

    #[test]
    fn add_member_rejects_duplicate_pair() {
        let conn = memory_db();
        users::insert(&conn, "alice", "Alice", "", "hash").unwrap();
        insert(&conn, "devs", "Devs", "").unwrap();
        add_member(&conn, "devs", "alice").unwrap();

        let err = add_member(&conn, "devs", "alice").unwrap_err();

        assert!(
            err.contains("cannot add alice to group devs"),
            "duplicate membership must fail loudly: {err}"
        );
    }

    #[test]
    fn add_member_rejects_unknown_user() {
        let conn = memory_db();
        insert(&conn, "devs", "Devs", "").unwrap();

        let err = add_member(&conn, "devs", "ghost").unwrap_err();

        assert!(
            err.contains("FOREIGN KEY constraint failed"),
            "membership of an unknown uid must be refused by the schema: {err}"
        );
    }
}
