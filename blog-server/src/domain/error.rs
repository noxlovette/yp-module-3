use crate::{domain::PasswordError, infra::JwtError};
use thiserror::Error;

pub type DomainResult<T> = Result<T, DomainError>;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Пользователь не найден")]
    UserNotFound,
    #[error("Пользователь с таким username уже есть")]
    UsernameAlreadyExists,
    #[error("Пользователь с таким email уже есть")]
    EmailAlreadyExists,
    #[error("Неверная пара логин/пароль")]
    InvalidCredentials,
    #[error("Пост не найден")]
    PostNotFound,
    #[error("Кышь отсюда")]
    Forbidden,
    #[error("Требуется авторизация")]
    Unauthorized,
    #[error("Expected list, got single, or vice versa")]
    TypeMismatch,
    #[error("validation/parsing error: {0}")]
    Parsing(#[from] ParsingError),
    #[error("password error: {0}")]
    Password(#[from] PasswordError),
    #[error("database error: {0}")]
    Database(String),
}

impl From<JwtError> for DomainError {
    // Deliberately collapse every JWT failure (bad signature, malformed
    // token, expired `exp`, ...) into one generic response. Distinguishing
    // them for the client would tell an attacker exactly which part of a
    // forged token to fix next.
    fn from(_value: JwtError) -> Self {
        DomainError::Unauthorized
    }
}

impl From<sqlx::migrate::MigrateError> for DomainError {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        DomainError::Database(value.to_string())
    }
}

impl From<sqlx::Error> for DomainError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => DomainError::PostNotFound,
            other => DomainError::Database(other.to_string()),
        }
    }
}

#[derive(Debug, Error)]
pub enum ParsingError {
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
