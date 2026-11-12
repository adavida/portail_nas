use rusqlite::{Connection, params};

pub fn remove_member(conn: &Connection, gid: &str, uid: &str) -> Result<usize, String> {
    conn.execute(
        "DELETE FROM group_members WHERE gid = ?1 AND uid = ?2",
        params![gid, uid],
    )
    .map_err(|e| format!("cannot remove {uid} from group {gid}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::groups::{add_member, insert, members_of};
    use crate::test_helpers::memory_db::memory_db;
    use crate::users;

    #[test]
    fn remove_member_deletes_only_the_pair() {
        let conn = memory_db();
        users::insert(&conn, "alice", "Alice", "", "hash").unwrap();
        users::insert(&conn, "bob", "Bob", "", "hash").unwrap();
        insert(&conn, "devs", "Devs", "").unwrap();
        add_member(&conn, "devs", "alice").unwrap();
        add_member(&conn, "devs", "bob").unwrap();

        let rows = remove_member(&conn, "devs", "alice").unwrap();

        assert_eq!(rows, 1, "one membership should be deleted");
        assert_eq!(
            members_of(&conn, "devs").unwrap(),
            vec!["bob".to_string()],
            "other member should remain"
        );
    }

    #[test]
    fn remove_member_returns_zero_for_unknown_pair() {
        let conn = memory_db();

        let rows = remove_member(&conn, "devs", "ghost").unwrap();

        assert_eq!(rows, 0, "unknown pair must affect no row");
    }
}
