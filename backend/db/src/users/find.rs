use rusqlite::Connection;

use super::record::UserRecord;

pub fn find_all(conn: &Connection) -> Result<Vec<UserRecord>, String> {
    let mut statement = conn
        .prepare("SELECT uid, name, email FROM users ORDER BY uid ASC")
        .map_err(|e| format!("cannot prepare user list: {e}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok(UserRecord {
                uid: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
            })
        })
        .map_err(|e| format!("cannot list users: {e}"))?;
    rows.collect::<Result<Vec<UserRecord>, _>>()
        .map_err(|e| format!("cannot read user rows: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::memory_db::memory_db;
    use crate::users::insert;

    #[test]
    fn find_all_returns_inserted_users_sorted_by_uid() {
        let conn = memory_db();
        insert(&conn, "bob", "Bob", "b@ex.com", "hash").unwrap();
        insert(&conn, "alice", "Alice", "", "hash").unwrap();

        let users = find_all(&conn).unwrap();

        assert_eq!(users.len(), 2, "both users should be listed");
        assert_eq!(
            users[0].uid, "alice",
            "rows should be sorted by uid ascending"
        );
        assert_eq!(users[0].name, "Alice", "name should roundtrip");
        assert_eq!(users[0].email, "", "empty email should roundtrip");
        assert_eq!(users[1].uid, "bob", "second row should be bob");
    }

    #[test]
    fn find_all_hides_password_hash() {
        let conn = memory_db();
        insert(&conn, "alice", "Alice", "", "topsecret-hash").unwrap();

        let users = find_all(&conn).unwrap();

        assert_eq!(
            users[0],
            UserRecord {
                uid: "alice".into(),
                name: "Alice".into(),
                email: "".into(),
            },
            "listed record must carry no password material"
        );
    }
}
