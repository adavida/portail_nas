use rusqlite::{Connection, params};
use std::os::unix::fs::PermissionsExt;

use crate::path::portal_db_path;
use portail_core::config::Config;

const DB_BUSY_TIMEOUT_SECS: u64 = 5;

const MIGRATIONS: &[(&str, &str)] = &[
    (
        "0001_user_opaque_identifier",
        include_str!("../migrations/0001_user_opaque_identifier.sql"),
    ),
    (
        "0002_directory_users_groups",
        include_str!("../migrations/0002_directory_users_groups.sql"),
    ),
];

pub fn connect(config: &Config) -> Result<Connection, String> {
    let path = portal_db_path(config);
    let parent = path
        .parent()
        .ok_or_else(|| format!("portail db path {} has no parent", path.display()))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("cannot create dir for {}: {e}", path.display()))?;
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(_) => std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("cannot set mode on {}: {e}", path.display()))?,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(format!("cannot create {}: {e}", path.display())),
    }
    let mut conn =
        Connection::open(&path).map_err(|e| format!("cannot open {}: {e}", path.display()))?;
    conn.busy_timeout(std::time::Duration::from_secs(DB_BUSY_TIMEOUT_SECS))
        .map_err(|e| format!("cannot set busy timeout on {}: {e}", path.display()))?;
    conn.pragma_update(None, "foreign_keys", true)
        .map_err(|e| format!("cannot enable foreign keys on {}: {e}", path.display()))?;
    init_schema(&mut conn).map_err(|e| format!("cannot init schema in {}: {e}", path.display()))?;
    Ok(conn)
}

pub(crate) fn init_schema(conn: &mut Connection) -> Result<(), rusqlite::Error> {
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    tx.execute_batch(
        "CREATE TABLE IF NOT EXISTS migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )?;
    for (name, sql) in MIGRATIONS {
        let applied = tx.query_row(
            "SELECT count(*) FROM migrations WHERE name = ?1",
            params![name],
            |row| row.get::<_, i64>(0),
        )? > 0;
        if applied {
            continue;
        }
        tx.execute_batch(sql)?;
        tx.execute("INSERT INTO migrations (name) VALUES (?1)", params![name])?;
    }
    tx.commit()
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
        let mut conn = crate::test_helpers::memory_db::memory_db();

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
            count as usize,
            MIGRATIONS.len(),
            "rerunning init_schema must not re-apply migrations"
        );
    }

    #[test]
    fn concurrent_connects_apply_each_migration_once() {
        let config = test_config("concurrent");
        let results = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..8)
                .map(|_| scope.spawn(|| connect(&config).map(|conn| conn.execute("SELECT 1", []))))
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().expect("thread must not panic"))
                .collect::<Vec<_>>()
        });

        assert!(
            results.iter().all(Result::is_ok),
            "every concurrent connect must succeed: {:?}",
            results.iter().map(|r| r.as_ref().err()).collect::<Vec<_>>()
        );
        let conn = connect(&config).unwrap();
        let count: i64 = conn
            .query_row("SELECT count(*) FROM migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(
            count as usize,
            MIGRATIONS.len(),
            "concurrent connects must record each migration exactly once"
        );
        let _ = std::fs::remove_dir_all(portal_db_path(&config).parent().unwrap());
    }
}
