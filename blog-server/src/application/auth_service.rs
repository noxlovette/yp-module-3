use std::sync::Arc;

use crate::{
    data::UserRepo,
    domain::{LoginPayload, ParsingError, SignupPayload, User},
    infra::{JwtError, JwtService, Token},
};
use serde::Serialize;
use sqlx::PgPool;
use thiserror::Error;

pub struct AuthService {
    repo: UserRepo,
    jwt: Arc<JwtService>,
}

#[derive(Debug, Error)]
pub enum AuthError {
    /// JWT-Related Errors
    #[error("JWT Error: {0}")]
    Jwt(#[from] JwtError),

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("validation/parsing error: {)}")]
    Parsing(#[from] ParsingError),
}

type AuthResult<T> = Result<T, AuthError>;

#[derive(Serialize, Debug)]
pub struct SignupResponse {
    token: Token,
    user: User,
}

impl AuthService {
    pub fn new(p: &PgPool) -> AuthResult<Arc<Self>> {
        Ok(Arc::new(Self {
            repo: UserRepo::new(p),
            jwt: JwtService::new()?,
        }))
    }

    /// Creates a new user
    pub fn signup(&self, p: SignupPayload) -> AuthResult<SignupResponse> {
        todo!()
    }

    /// Compares the given password with the one stored in the db
    ///
    /// Issues a JWT on success
    pub fn login(&self, p: LoginPayload) -> AuthResult<()> {
        todo!()
    }
}
