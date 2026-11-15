use rusqlite::{Connection, TransactionBehavior, params};

pub fn ensure_labels(conn: &mut Connection, names: &[String]) -> Result<(), String> {
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| format!("cannot open label transaction: {e}"))?;
    for name in names {
        let known: i64 = tx
            .query_row(
                "SELECT count(*) FROM attr_labels WHERE name = ?1",
                params![name],
                |row| row.get(0),
            )
            .map_err(|e| format!("cannot count attr label {name}: {e}"))?;
        if known == 0 {
            tx.execute("INSERT INTO attr_labels (name) VALUES (?1)", params![name])
                .map_err(|e| format!("cannot insert attr label {name}: {e}"))?;
        }
    }
    tx.commit()
        .map_err(|e| format!("cannot commit attr labels: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::memory_db::memory_db;

    #[test]
    fn ensure_labels_inserts_missing_names_once() {
        let mut conn = memory_db();

        ensure_labels(&mut conn, &["phone".into(), "phone".into(), "city".into()]).unwrap();
        ensure_labels(&mut conn, &["phone".into()]).unwrap();

        let count: i64 = conn
            .query_row("SELECT count(*) FROM attr_labels", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 2, "each distinct name must be stored exactly once");
    }
}
