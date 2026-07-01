use crate::{
    data::{LoginCaller, SignupDb, UserDb},
    domain::{ParsingError, user},
};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::SaltString,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, de};
use std::sync::LazyLock;
use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct Username(String);

impl From<&str> for Username {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct Email(String);

#[derive(Debug, Clone)]
pub enum Password {
    Plain(String),
    Hashed(String),
}

impl Into<String> for Email {
    fn into(self) -> String {
        self.0
    }
}

impl Into<String> for Username {
    fn into(self) -> String {
        self.0
    }
}

impl TryInto<SignupDb> for SignupPayload {
    type Error = PasswordError;
    fn try_into(self) -> Result<SignupDb, Self::Error> {
        Ok(SignupDb {
            username: self.username.into(),
            email: self.email.into(),
            password_hash: self.password.get_hash()?,
        })
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: Username,
    pub email: Email,
    #[serde(skip_serializing)]
    pub password_hash: Password,
    pub created_at: DateTime<Utc>,
}
impl From<UserDb> for User {
    fn from(value: UserDb) -> Self {
        let UserDb {
            id,
            username,
            email,
            password_hash,
            created_at,
        } = value;

        Self {
            id,
            username: Username::new(username),
            email: Email::new(email),
            password_hash: Password::new_hashed(password_hash),
            created_at,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct SignupPayload {
    pub username: Username,
    pub password: Password,
    pub email: Email,
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

impl LoginPayload {
    pub fn get_password(&self) -> &Password {
        match self {
            Self::Email { password, .. } => password,
            Self::Username { password, .. } => password,
        }
    }
}

impl<'a> Into<LoginCaller<'a>> for &'a LoginPayload {
    fn into(self) -> LoginCaller<'a> {
        match self {
            LoginPayload::Email { email, .. } => LoginCaller::Email(email),
            LoginPayload::Username { username, .. } => {
                LoginCaller::Username(username)
            }
        }
    }
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

impl<'de> Deserialize<'de> for Username {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?)
            .map_err(de::Error::custom)
    }
}

pub static ARGON: LazyLock<Argon2> =
    LazyLock::new(|| argon2::Argon2::default());

impl Password {
    const MAX: usize = 32;
    const MIN: usize = 8;

    /// Creates a Hashed instance of Password
    pub fn new_hashed(s: impl Into<String>) -> Self {
        Self::Hashed(s.into())
    }

    /// Creates a Plain instance of Password
    pub fn new_plain(s: impl Into<String>) -> Self {
        Self::Plain(s.into())
    }

    pub fn get_hash(self) -> Result<String, PasswordError> {
        match self {
            Self::Hashed(h) => Ok(h),
            _ => Err(PasswordError::HashRequired),
        }
    }

    /// Parses a string into a plain password
    fn parse(raw: String) -> Result<Self, ParsingError> {
        if !(Self::MIN..=Self::MAX).contains(&raw.graphemes(true).count()) {
            return Err(ParsingError::InvalidLength {
                entity: "password",
                min: Self::MIN,
                max: Self::MAX,
            });
        }

        Ok(Self::new_plain(raw))
    }

    /// Hashes a password. Argon2
    pub fn hash(self) -> Result<Self, PasswordError> {
        match self {
            Password::Plain(p) => {
                let salt = SaltString::generate(
                    &mut argon2::password_hash::rand_core::OsRng,
                );

                Ok(Self::new_hashed(
                    ARGON.hash_password(p.as_bytes(), &salt)?.to_string(),
                ))
            }
            _ => Err(PasswordError::PlainRequired),
        }
    }

    /// Validates a password. Should be applied to Plain versions only. The
    /// other should be hashed
    pub fn validate(&self, other: &Password) -> Result<(), PasswordError> {
        match self {
            Self::Hashed(_) => Err(PasswordError::PlainRequired),
            Self::Plain(p) => match other {
                Self::Hashed(o) => ARGON
                    .verify_password(p.as_bytes(), &PasswordHash::new(o)?)
                    .map_err(Into::into),
                Self::Plain(_) => Err(PasswordError::HashRequired),
            },
        }
    }
}

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("Plain string passed where hash was expected")]
    HashRequired,
    #[error("Hashed string passed where plain was expected")]
    PlainRequired,
    #[error("hashing/verification error: {0}")]
    HashingFailed(String),
}

impl From<argon2::password_hash::Error> for PasswordError {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::HashingFailed(value.to_string())
    }
}

impl<'de> Deserialize<'de> for Password {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?)
            .map_err(de::Error::custom)
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
        Self::parse(String::deserialize(deserializer)?)
            .map_err(de::Error::custom)
    }
}

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
