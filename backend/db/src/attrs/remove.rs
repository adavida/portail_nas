use rusqlite::{Connection, params};

use super::label_id;

pub fn delete_attr(conn: &Connection, uid: &str, label: &str) -> Result<(), String> {
    let Some(id) = label_id(conn, label)? else {
        return Ok(());
    };
    conn.execute(
        "DELETE FROM user_attrs WHERE user_id = ?1 AND label_id = ?2",
        params![uid, id],
    )
    .map_err(|e| format!("cannot delete attr {label} of user {uid}: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attrs::{list_attrs, set_attr};
    use crate::test_helpers::memory_db::memory_db;
    use crate::users::insert;

    #[test]
    fn delete_attr_removes_only_the_pair() {
        let conn = memory_db();
        insert(&conn, "alice", "A", "", "h").unwrap();
        set_attr(&conn, "alice", "city", "Paris").unwrap();
        set_attr(&conn, "alice", "phone", "0600").unwrap();

        delete_attr(&conn, "alice", "city").unwrap();
        delete_attr(&conn, "alice", "never-known").unwrap();

        assert_eq!(
            list_attrs(&conn, "alice").unwrap(),
            vec![("phone".to_string(), "0600".to_string())],
            "only the deleted label must disappear, unknown labels are no-ops"
        );
    }
}
