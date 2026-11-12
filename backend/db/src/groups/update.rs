use rusqlite::{Connection, params};

pub fn update(
    conn: &Connection,
    gid: &str,
    name: &str,
    description: &str,
) -> Result<usize, String> {
    conn.execute(
        "UPDATE groups SET name = ?2, description = ?3 WHERE gid = ?1",
        params![gid, name, description],
    )
    .map_err(|e| format!("cannot update group {gid}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::groups::{find_all, insert};
    use crate::test_helpers::memory_db::memory_db;

    #[test]
    fn update_changes_name_and_description() {
        let conn = memory_db();
        insert(&conn, "devs", "Old", "old desc").unwrap();

        let rows = update(&conn, "devs", "New", "").unwrap();

        assert_eq!(rows, 1, "one row should be updated");
        let groups = find_all(&conn).unwrap();
        assert_eq!(groups[0].name, "New", "name should be updated");
        assert_eq!(
            groups[0].description, "",
            "empty description should clear the stored one"
        );
    }

    #[test]
    fn update_returns_zero_for_unknown_gid() {
        let conn = memory_db();

        let rows = update(&conn, "ghost", "New", "").unwrap();

        assert_eq!(rows, 0, "unknown gid must affect no row");
    }
}
