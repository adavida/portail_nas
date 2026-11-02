use rusqlite::{Connection, params};

pub fn insert(
    conn: &Connection,
    client_id: &str,
    username: &str,
    identifier: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO user_opaque_identifier (client_id, username, identifier) VALUES (?1, ?2, ?3)",
        params![client_id, username, identifier],
    )
    .map_err(|e| format!("cannot save subject for {client_id}/{username}: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connect::init_schema;
    use crate::subjects::find;

    fn memory_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        init_schema(&mut conn).unwrap();
        conn
    }

    #[test]
    fn insert_then_find_roundtrips_the_subject() {
        let conn = memory_db();

        insert(&conn, "client-a", "alice", "uuid-1").unwrap();

        assert_eq!(
            find(&conn, "client-a", "alice").unwrap().as_deref(),
            Some("uuid-1"),
            "insert must persist the subject for the same signature"
        );
    }

    #[test]
    fn insert_rejects_duplicate_signature() {
        let conn = memory_db();
        insert(&conn, "client-a", "alice", "uuid-1").unwrap();

        let err = insert(&conn, "client-a", "alice", "uuid-2").unwrap_err();

        assert!(
            err.contains("cannot save subject"),
            "duplicate primary key must fail loudly: {err}"
        );
    }
}
