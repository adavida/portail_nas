use rusqlite::{Connection, OptionalExtension, params};

pub fn find(conn: &Connection, client_id: &str, username: &str) -> Result<Option<String>, String> {
    conn.query_row(
        "SELECT identifier FROM user_opaque_identifier WHERE client_id = ?1 AND username = ?2",
        params![client_id, username],
        |row| row.get(0),
    )
    .optional()
    .map_err(|e| format!("cannot query subject for {client_id}/{username}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connect::init_schema;
    use crate::subjects::insert;

    fn memory_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        init_schema(&mut conn).unwrap();
        conn
    }

    #[test]
    fn find_returns_none_for_unknown_signature() {
        let conn = memory_db();

        let found = find(&conn, "client-a", "ghost").unwrap();

        assert!(found.is_none(), "unknown client+user must return None");
    }

    #[test]
    fn find_returns_the_stored_identifier() {
        let conn = memory_db();
        insert(&conn, "client-a", "alice", "uuid-alice").unwrap();

        let found = find(&conn, "client-a", "alice").unwrap();

        assert_eq!(
            found.as_deref(),
            Some("uuid-alice"),
            "find must return the stored pairwise subject"
        );
    }
}
