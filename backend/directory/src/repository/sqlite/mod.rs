pub mod groups;
pub mod users;

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use portail_core::error::AppError;

pub(crate) fn hash_password(password: &str) -> Result<String, AppError> {
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::Db(format!("cannot hash password: {e}")))
}

pub(crate) fn verify_password(stored_hash: &str, password: &str) -> bool {
    PasswordHash::new(stored_hash).ok().is_some_and(|parsed| {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_then_verify_roundtrip() {
        let hash = hash_password("s3cret").unwrap();

        assert!(
            hash.starts_with("$argon2"),
            "stored form should be an argon2 PHC string, got {hash}"
        );
        assert!(
            verify_password(&hash, "s3cret"),
            "correct password should verify against its own hash"
        );
        assert!(
            !verify_password(&hash, "wrong"),
            "wrong password should not verify"
        );
    }

    #[test]
    fn hash_password_uses_fresh_salt_each_time() {
        let first = hash_password("s3cret").unwrap();
        let second = hash_password("s3cret").unwrap();

        assert_ne!(
            first, second,
            "identical passwords must not produce identical hashes"
        );
        assert!(
            verify_password(&first, "s3cret") && verify_password(&second, "s3cret"),
            "both salts must verify back"
        );
    }

    #[test]
    fn verify_password_rejects_garbage_hash() {
        assert!(
            !verify_password("not-a-hash", "s3cret"),
            "an unparseable stored hash must fail closed"
        );
    }
}
