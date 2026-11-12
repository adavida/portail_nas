use rusqlite::Connection;

use crate::connect::init_schema;

#[cfg(test)]
pub(crate) fn memory_db() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", true)
        .expect("in-memory db must accept the foreign_keys pragma");
    init_schema(&mut conn).unwrap();
    conn
}
