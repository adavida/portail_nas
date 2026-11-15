mod ensure_labels;
mod list;
mod remove;
mod set;

pub use ensure_labels::ensure_labels;
pub use list::list_attrs;
pub use remove::delete_attr;
pub use set::set_attr;

use rusqlite::{Connection, OptionalExtension};

pub(crate) fn label_id(conn: &Connection, label: &str) -> Result<Option<i64>, String> {
    conn.query_row(
        "SELECT id FROM attr_labels WHERE name = ?1",
        rusqlite::params![label],
        |row| row.get(0),
    )
    .optional()
    .map_err(|e| format!("cannot look up attr label {label}: {e}"))
}
