use portail_core::config::{AttrKind, AttrSpec, Config};
use serde::Deserialize;

use crate::domain::ldap::Email;
use errors::AttrError;

pub mod errors;

pub const LABEL_MAX_CHARS: usize = 64;
pub const VALUE_MAX_CHARS: usize = 500;
pub const TELEPHONE_MIN_CHARS: usize = 6;
pub const TELEPHONE_MAX_CHARS: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserAttr {
    pub label: String,
    pub value: String,
}

impl UserAttr {
    pub fn try_new(label: String, value: String) -> Result<Self, AttrError> {
        let label = label.trim().to_string();
        if label.is_empty() {
            return Err(AttrError::EmptyLabel);
        }
        if label.chars().count() > LABEL_MAX_CHARS || !label.chars().all(Self::label_char_ok) {
            return Err(AttrError::InvalidLabel(label));
        }
        let spec = Config::global().attr_spec(&label);
        let value = value.trim().to_string();
        Self::check_value(&label, &value, spec)?;
        Ok(Self { label, value })
    }

    pub fn from_stored(label: String, value: String) -> Self {
        Self { label, value }
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    fn label_char_ok(c: char) -> bool {
        c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-')
    }

    fn check_value(label: &str, value: &str, spec: Option<&AttrSpec>) -> Result<(), AttrError> {
        if value.is_empty() {
            return Ok(());
        }
        let Some(spec) = spec else {
            if value.chars().count() > VALUE_MAX_CHARS {
                return Err(AttrError::TooLong {
                    label: label.to_string(),
                    max: VALUE_MAX_CHARS,
                });
            }
            return Ok(());
        };
        match spec.kind {
            AttrKind::Text => {
                let max = spec.max_chars.unwrap_or(VALUE_MAX_CHARS);
                if value.chars().count() > max {
                    return Err(AttrError::TooLong {
                        label: label.to_string(),
                        max,
                    });
                }
            }
            AttrKind::Number => {
                let parsed = value
                    .parse::<i64>()
                    .map_err(|_| AttrError::NotANumber(label.to_string()))?;
                if spec.min.is_some_and(|min| parsed < min)
                    || spec.max.is_some_and(|max| parsed > max)
                {
                    return Err(AttrError::NumberOutOfRange {
                        label: label.to_string(),
                        min: spec.min,
                        max: spec.max,
                    });
                }
            }
            AttrKind::Email => {
                Email::try_new(value.to_string())
                    .map_err(|_| AttrError::InvalidEmail(label.to_string()))?;
            }
            AttrKind::Telephone => {
                let chars = value.chars().count();
                if !(TELEPHONE_MIN_CHARS..=TELEPHONE_MAX_CHARS).contains(&chars)
                    || !value.chars().all(|c| {
                        c.is_ascii_digit() || matches!(c, '+' | ' ' | '(' | ')' | '.' | '/' | '-')
                    })
                {
                    return Err(AttrError::InvalidTelephone(label.to_string()));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attr(label: &str, value: &str) -> Result<UserAttr, AttrError> {
        UserAttr::try_new(label.to_string(), value.to_string())
    }

    #[test]
    fn accepts_value_within_text_limit() {
        let result = attr("city", "Paris");

        assert_eq!(
            result,
            Ok(UserAttr {
                label: "city".into(),
                value: "Paris".into()
            }),
            "a catalogued text value should pass"
        );
    }

    #[test]
    fn rejects_text_value_above_max_chars() {
        let long = "a".repeat(51);

        let err = attr("city", &long).unwrap_err();

        assert!(
            matches!(&err, AttrError::TooLong { label, max } if label == "city" && *max == 50),
            "text over its max_chars should be rejected with its own limit, got {err:?}"
        );
    }

    #[test]
    fn accepts_number_in_range_and_rejects_outside() {
        assert!(attr("age", "42").is_ok(), "42 is within 0..150");

        let err = attr("age", "200").unwrap_err();
        assert!(
            matches!(&err, AttrError::NumberOutOfRange { .. }),
            "200 is above the max bound, got {err:?}"
        );

        let err = attr("age", "abc").unwrap_err();
        assert!(
            matches!(&err, AttrError::NotANumber(l) if l == "age"),
            "'abc' is not a number, got {err:?}"
        );
    }

    #[test]
    fn accepts_valid_email_and_rejects_garbage() {
        assert!(
            attr("contact", "a@b.com").is_ok(),
            "valid email should pass"
        );

        let err = attr("contact", "nope").unwrap_err();
        assert!(
            matches!(&err, AttrError::InvalidEmail(l) if l == "contact"),
            "an invalid email should be rejected, got {err:?}"
        );
    }

    #[test]
    fn accepts_common_telephone_formats_and_rejects_letters() {
        assert!(attr("telephone", "+33 6 12 34 56 78").is_ok());
        assert!(attr("telephone", "01.23.45.67.89").is_ok());

        let err = attr("telephone", "06-abc-12").unwrap_err();
        assert!(
            matches!(&err, AttrError::InvalidTelephone(l) if l == "telephone"),
            "letters must not pass the telephone check, got {err:?}"
        );

        let err = attr("telephone", "12345").unwrap_err();
        assert!(
            matches!(err, AttrError::InvalidTelephone(_)),
            "below the minimum length should be rejected, got {err:?}"
        );
    }

    #[test]
    fn empty_value_is_accepted_as_deletion() {
        assert!(
            attr("city", "").is_ok(),
            "an empty value means removal and must pass validation"
        );
    }

    #[test]
    fn unlisted_label_gets_the_default_length_cap() {
        let long = "x".repeat(VALUE_MAX_CHARS + 1);

        let err = attr("inconnu", &long).unwrap_err();
        assert!(
            matches!(&err, AttrError::TooLong { label, max } if label == "inconnu" && *max == VALUE_MAX_CHARS),
            "unlisted labels fall back to the generic cap, got {err:?}"
        );

        assert!(attr("inconnu", "short").is_ok());
    }

    #[test]
    fn rejects_bad_label_names() {
        assert!(
            matches!(attr("", "x"), Err(AttrError::EmptyLabel)),
            "an empty label must be rejected"
        );
        assert!(
            matches!(attr("Bad Label", "x"), Err(AttrError::InvalidLabel(_))),
            "uppercase and spaces are not valid label characters"
        );
    }
}
