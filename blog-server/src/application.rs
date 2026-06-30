mod auth_service;
mod blog_service;

pub use auth_service::*;
pub use blog_service::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Auth Error")]
    Auth(#[from] AuthError),

    #[error("Expected list, got single, or vice versa")]
    TypeMismatch,
}

pub type ApplicationResult<T> = Result<T, ApplicationError>;
