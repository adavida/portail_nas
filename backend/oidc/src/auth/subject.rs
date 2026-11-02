use crate::auth::store;
use portail_core::config::Config;
use portail_db::connect;
use portail_db::subjects;

pub fn subject_for(config: &Config, client_id: &str, username: &str) -> Result<String, String> {
    let conn = connect(config)?;
    if let Some(subject) = subjects::find(&conn, client_id, username)? {
        return Ok(subject);
    }
    let subject = uuid_v4()?;
    subjects::insert(&conn, client_id, username, &subject)?;
    Ok(subject)
}

fn uuid_v4() -> Result<String, String> {
    let hex = store::random_id()?;
    let mut bytes: Vec<u8> = (0..16)
        .map(|i| {
            u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
                .map_err(|e| format!("cannot parse random hex: {e}"))
        })
        .collect::<Result<_, _>>()?;
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let all: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &all[0..8],
        &all[8..12],
        &all[12..16],
        &all[16..20],
        &all[20..32]
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config(tag: &str) -> Config {
        let mut config = Config::global().clone();
        config.db_path = format!(
            "{}/portail-subject-test-{tag}-{}/portail.db",
            std::env::temp_dir().display(),
            std::process::id()
        );
        config
    }

    #[test]
    fn subject_for_returns_stable_uuid_v4_per_client_and_user() {
        let config = test_config("stable");
        let first = subject_for(&config, "client-a", "alice").unwrap();
        let second = subject_for(&config, "client-a", "alice").unwrap();
        let other_user = subject_for(&config, "client-a", "bob").unwrap();
        let other_client = subject_for(&config, "client-b", "alice").unwrap();

        assert_eq!(first, second, "same client+user must reuse the subject");
        assert_eq!(first.len(), 36, "subject must be a dashed uuid: {first}");
        assert_eq!(&first[14..15], "4", "uuid version nibble must be 4");
        assert!(
            ["8", "9", "a", "b"].contains(&&first[19..20]),
            "uuid variant nibble must be 8-b: {first}"
        );
        assert!(!first.contains("alice"), "subject must not leak username");
        assert_ne!(first, other_user, "distinct users get distinct subjects");
        assert_ne!(
            first, other_client,
            "pairwise: distinct clients get distinct subjects"
        );
        let db = std::path::Path::new(&config.db_path);
        assert!(db.exists(), "portail db must be created");
        let _ = std::fs::remove_dir_all(db.parent().unwrap());
    }

    #[test]
    fn subject_survives_reopening_database() {
        let config = test_config("reopen");
        let first = subject_for(&config, "client-a", "alice").unwrap();
        let reopened = subject_for(&config, "client-a", "alice").unwrap();

        assert_eq!(
            first, reopened,
            "subject must persist across fresh connections"
        );
        let _ = std::fs::remove_dir_all(std::path::Path::new(&config.db_path).parent().unwrap());
    }

    #[test]
    fn uuid_v4_shape_and_uniqueness() {
        let one = uuid_v4().unwrap();
        let two = uuid_v4().unwrap();

        assert_eq!(one.len(), 36, "uuid must be 8-4-4-4-12");
        assert!(
            one.chars().filter(|c| *c == '-').count() == 4,
            "uuid must have 4 dashes: {one}"
        );
        assert_ne!(one, two, "consecutive uuids must differ");
    }
}
