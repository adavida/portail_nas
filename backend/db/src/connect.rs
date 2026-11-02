use rusqlite::{Connection, params};
use std::os::unix::fs::PermissionsExt;

use crate::path::portal_db_path;
use portail_core::config::Config;

const MIGRATIONS: &[(&str, &str)] = &[(
    "0001_user_opaque_identifier",
    include_str!("../migrations/0001_user_opaque_identifier.sql"),
)];

pub fn connect(config: &Config) -> Result<Connection, String> {
    let path = portal_db_path(config);
    let parent = path
        .parent()
        .ok_or_else(|| format!("portail db path {} has no parent", path.display()))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("cannot create dir for {}: {e}", path.display()))?;
    if !path.exists() {
        std::fs::write(&path, []).map_err(|e| format!("cannot create {}: {e}", path.display()))?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("cannot set mode on {}: {e}", path.display()))?;
    }
    let mut conn =
        Connection::open(&path).map_err(|e| format!("cannot open {}: {e}", path.display()))?;
    init_schema(&mut conn).map_err(|e| format!("cannot init schema in {}: {e}", path.display()))?;
    Ok(conn)
}

pub(crate) fn init_schema(conn: &mut Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )?;
    for (name, sql) in MIGRATIONS {
        let applied = conn.query_row(
            "SELECT count(*) FROM migrations WHERE name = ?1",
            params![name],
            |row| row.get::<_, i64>(0),
        )? > 0;
        if applied {
            continue;
        }
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.execute("INSERT INTO migrations (name) VALUES (?1)", params![name])?;
        tx.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn test_config(tag: &str) -> Config {
        let mut config = Config::global().clone();
        config.db_path = format!(
            "{}/portail-db-test-{tag}-{}/portail.db",
            std::env::temp_dir().display(),
            std::process::id()
        );
        config
    }

    #[test]
    fn connect_creates_file_with_schema_and_permissions() {
        let config = test_config("create");

        let conn = connect(&config).unwrap();
        let count: i64 = conn
            .query_row("SELECT count(*) FROM user_opaque_identifier", [], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(count, 0, "fresh db must expose an empty subject table");
        let db = portal_db_path(&config);
        assert!(db.exists(), "connect must create the db file");
        assert_eq!(
            std::fs::metadata(&db).unwrap().permissions().mode() & 0o777,
            0o600,
            "db file must be 0600"
        );
        drop(conn);
        let again = connect(&config);
        assert!(again.is_ok(), "connect must be idempotent on existing db");
        let _ = std::fs::remove_dir_all(db.parent().unwrap());
    }

    #[test]
    fn init_schema_records_each_migration_exactly_once() {
        let mut conn = Connection::open_in_memory().unwrap();

        init_schema(&mut conn).unwrap();
        init_schema(&mut conn).unwrap();

        let applied: String = conn
            .query_row("SELECT name FROM migrations LIMIT 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(
            applied, "0001_user_opaque_identifier",
            "first migration must be recorded by name"
        );
        let count: i64 = conn
            .query_row("SELECT count(*) FROM migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(
            count, 1,
            "rerunning init_schema must not re-apply migrations"
        );
    }
}
