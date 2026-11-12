use rusqlite::{Connection, params};

pub fn members_of(conn: &Connection, gid: &str) -> Result<Vec<String>, String> {
    let mut statement = conn
        .prepare("SELECT uid FROM group_members WHERE gid = ?1 ORDER BY uid ASC")
        .map_err(|e| format!("cannot prepare member list: {e}"))?;
    let rows = statement
        .query_map(params![gid], |row| row.get::<_, String>(0))
        .map_err(|e| format!("cannot list members of {gid}: {e}"))?;
    rows.collect::<Result<Vec<String>, _>>()
        .map_err(|e| format!("cannot read member rows of {gid}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::groups::{add_member, insert};
    use crate::test_helpers::memory_db::memory_db;
    use crate::users;

    #[test]
    fn members_of_returns_only_that_group_sorted() {
        let conn = memory_db();
        users::insert(&conn, "alice", "Alice", "", "hash").unwrap();
        users::insert(&conn, "bob", "Bob", "", "hash").unwrap();
        users::insert(&conn, "carol", "Carol", "", "hash").unwrap();
        insert(&conn, "devs", "Devs", "").unwrap();
        insert(&conn, "ops", "Ops", "").unwrap();
        add_member(&conn, "devs", "bob").unwrap();
        add_member(&conn, "devs", "alice").unwrap();
        add_member(&conn, "ops", "carol").unwrap();

        let members = members_of(&conn, "devs").unwrap();

        assert_eq!(
            members,
            vec!["alice".to_string(), "bob".to_string()],
            "only devs members, sorted, expected"
        );
    }
}
