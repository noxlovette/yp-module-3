use crate::{
    data::UserRepo,
    infra::{JwtError, JwtService},
};
use sqlx::PgPool;
use thiserror::Error;

pub struct AuthService {
    repo: UserRepo,
    jwt: JwtService,
}

#[derive(Debug, Error)]
pub enum AuthError {
    /// JWT-Related Errors
    #[error("JWT Error: {0}")]
    Jwt(#[from] JwtError),
}

type AuthResult<T> = Result<T, AuthError>;

impl AuthService {
    pub fn new(p: &PgPool) -> AuthResult<Self> {
        Ok(Self {
            repo: UserRepo::new(p),
            jwt: JwtService::new()?,
        })
    }

    /// Creates a new user
    pub fn signup() -> AuthResult<()> {
        todo!()
    }

    /// Compares the given password with the one stored in the db
    ///
    /// Issues a JWT on success
    pub fn login() -> AuthResult<()> {
        todo!()
    }
}
