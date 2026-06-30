mod error;
mod post;
mod user;

pub use error::*;
pub use post::*;
pub use user::*;

#[derive(Debug, thiserror::Error)]
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
