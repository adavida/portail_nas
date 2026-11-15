use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrError {
    EmptyLabel,
    InvalidLabel(String),
    TooLong {
        label: String,
        max: usize,
    },
    NotANumber(String),
    NumberOutOfRange {
        label: String,
        min: Option<i64>,
        max: Option<i64>,
    },
    InvalidEmail(String),
    InvalidTelephone(String),
}

impl fmt::Display for AttrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyLabel => write!(f, "attr label is empty"),
            Self::InvalidLabel(l) => write!(
                f,
                "invalid attr label '{l}' (allowed: lowercase letters, digits, '.', '_', '-')"
            ),
            Self::TooLong { label, max } => {
                write!(f, "attr '{label}' value exceeds {max} characters")
            }
            Self::NotANumber(l) => write!(f, "attr '{l}' value must be a whole number"),
            Self::NumberOutOfRange { label, min, max } => match (min, max) {
                (Some(min), Some(max)) => {
                    write!(f, "attr '{label}' value must be between {min} and {max}")
                }
                (Some(min), None) => write!(f, "attr '{label}' value must be at least {min}"),
                (None, Some(max)) => write!(f, "attr '{label}' value must be at most {max}"),
                (None, None) => write!(f, "attr '{label}' value is out of range"),
            },
            Self::InvalidEmail(l) => write!(f, "attr '{l}' value must be a valid email"),
            Self::InvalidTelephone(l) => {
                write!(f, "attr '{l}' value must be a valid telephone number")
            }
        }
    }
}

impl std::error::Error for AttrError {}
