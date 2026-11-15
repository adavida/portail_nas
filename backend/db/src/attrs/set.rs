use rusqlite::{Connection, params};

use super::label_id;

pub fn set_attr(conn: &Connection, uid: &str, label: &str, value: &str) -> Result<(), String> {
    let id = match label_id(conn, label)? {
        Some(id) => id,
        None => {
            conn.execute("INSERT INTO attr_labels (name) VALUES (?1)", params![label])
                .map_err(|e| format!("cannot insert attr label {label}: {e}"))?;
            label_id(conn, label)?.ok_or_else(|| format!("attr label {label} vanished"))?
        }
    };
    let updated = conn
        .execute(
            "UPDATE user_attrs SET value = ?3 WHERE user_id = ?1 AND label_id = ?2",
            params![uid, id, value],
        )
        .map_err(|e| format!("cannot update attr {label} of user {uid}: {e}"))?;
    if updated == 0 {
        conn.execute(
            "INSERT INTO user_attrs (user_id, label_id, value) VALUES (?1, ?2, ?3)",
            params![uid, id, value],
        )
        .map_err(|e| format!("cannot insert attr {label} for user {uid}: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::memory_db::memory_db;
    use crate::users::insert;

    #[test]
    fn set_attr_inserts_then_updates() {
        let conn = memory_db();
        insert(&conn, "alice", "A", "", "h").unwrap();

        set_attr(&conn, "alice", "city", "Paris").unwrap();
        set_attr(&conn, "alice", "city", "Lyon").unwrap();

        let value: String = conn
            .query_row(
                "SELECT a.value FROM user_attrs a JOIN attr_labels l ON l.id = a.label_id
                 WHERE a.user_id = 'alice' AND l.name = 'city'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(value, "Lyon", "second write must replace the first");
        let count: i64 = conn
            .query_row("SELECT count(*) FROM user_attrs", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1, "one row per user and label");
    }

    #[test]
    fn set_attr_fails_for_unknown_user() {
        let conn = memory_db();

        let err = set_attr(&conn, "ghost", "city", "Paris").unwrap_err();

        assert!(
            err.contains("cannot insert attr"),
            "the foreign key on users must reject unknown uids: {err}"
        );
    }
}
