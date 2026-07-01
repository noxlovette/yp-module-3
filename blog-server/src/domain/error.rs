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
    #[error("Expected list, got single, or vice versa")]
    TypeMismatch,
    #[error("validation/parsing error: {0}")]
    Parsing(#[from] ParsingError),
    #[error("password error: {0}")]
    Password(#[from] PasswordError),
}

impl From<JwtError> for DomainError {
    fn from(value: JwtError) -> Self {
        todo!()
    }
}

impl From<sqlx::Error> for DomainError {
    fn from(value: sqlx::Error) -> Self {
        todo!()
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
