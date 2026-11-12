use rusqlite::Connection;

use super::record::GroupRecord;

pub fn find_all(conn: &Connection) -> Result<Vec<GroupRecord>, String> {
    let mut statement = conn
        .prepare("SELECT gid, name, description FROM groups ORDER BY gid ASC")
        .map_err(|e| format!("cannot prepare group list: {e}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok(GroupRecord {
                gid: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
            })
        })
        .map_err(|e| format!("cannot list groups: {e}"))?;
    rows.collect::<Result<Vec<GroupRecord>, _>>()
        .map_err(|e| format!("cannot read group rows: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::groups::insert;
    use crate::test_helpers::memory_db::memory_db;

    #[test]
    fn find_all_returns_inserted_groups_sorted() {
        let conn = memory_db();
        insert(&conn, "ops", "Ops", "operations").unwrap();
        insert(&conn, "devs", "Devs", "").unwrap();

        let groups = find_all(&conn).unwrap();

        assert_eq!(groups.len(), 2, "both groups should be listed");
        assert_eq!(groups[0].gid, "devs", "rows sorted by gid");
        assert_eq!(groups[0].name, "Devs", "name should roundtrip");
        assert_eq!(groups[0].description, "", "empty description roundtrips");
        assert_eq!(
            groups[1].description, "operations",
            "description should roundtrip"
        );
    }
}
