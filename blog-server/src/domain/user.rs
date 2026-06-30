use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, de};
use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
struct Username(String);

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
struct Email(String);

#[derive(Debug, Clone)]
struct Password(String);

#[derive(Serialize, Debug, Clone)]
pub struct User {
    id: i64,
    username: Username,
    email: Email,
    #[serde(skip_serializing)]
    password_hash: Password,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SignupPayload {
    username: Username,
    password: Password,
    email: Email,
}

#[derive(Deserialize, Debug, Clone)]
pub enum LoginPayload {
    Username {
        username: Username,
        password: Password,
    },
    Email {
        email: Email,
        password: Password,
    },
}

impl Username {
    const MAX: usize = 32;
    const MIN: usize = 2;

    /// Creates a new username
    pub fn new(u: impl Into<String>) -> Self {
        Self(u.into())
    }

    /// Parses passed string to validate length and chars used in the username
    fn parse(raw: String) -> Result<Self, ParsingError> {
        if !(Self::MIN..=Self::MAX).contains(&raw.graphemes(true).count()) {
            return Err(ParsingError::InvalidLength {
                entity: "username",
                min: Self::MIN,
                max: Self::MAX,
            });
        }

        if raw.chars().any(char::is_whitespace) {
            return Err(ParsingError::InvalidChar("username"));
        }

        Ok(Self(raw))
    }
}

#[derive(Debug, Error)]
enum ParsingError {
    #[error("{entity} must be between {min} and {max} characters")]
    InvalidLength {
        entity: &'static str,
        min: usize,
        max: usize,
    },

    #[error("{0} containes invalid char")]
    InvalidChar(&'static str),

    #[error("invalid format for {0}")]
    InvalidFormat(&'static str),
}

impl<'de> Deserialize<'de> for Username {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

impl Password {
    const MAX: usize = 32;
    const MIN: usize = 8;

    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    fn parse(raw: String) -> Result<Self, ParsingError> {
        if !(Self::MIN..=Self::MAX).contains(&raw.graphemes(true).count()) {
            return Err(ParsingError::InvalidLength {
                entity: "password",
                min: Self::MIN,
                max: Self::MAX,
            });
        }

        Ok(Self(raw))
    }
}

impl<'de> Deserialize<'de> for Password {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

impl Email {
    pub fn new(e: impl Into<String>) -> Self {
        Self(e.into())
    }

    pub fn parse(raw: String) -> Result<Self, ParsingError> {
        if email_address::EmailAddress::is_valid(&raw) {
            Ok(Self(raw))
        } else {
            Err(ParsingError::InvalidFormat("email"))
        }
    }
}

impl<'de> Deserialize<'de> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}
