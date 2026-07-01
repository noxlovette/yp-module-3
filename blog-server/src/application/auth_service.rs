use std::sync::Arc;

use crate::{
    data::{SignupDb, UserRepo},
    domain::{DomainResult, LoginPayload, SignupPayload, User},
    infra::{JwtService, Token},
};
use serde::Serialize;
use sqlx::PgPool;

pub struct AuthService {
    repo: UserRepo,
    jwt: Arc<JwtService>,
}

#[derive(Serialize, Debug)]
pub struct SignupResponse {
    token: Token,
    user: User,
}

impl From<(Token, User)> for SignupResponse {
    fn from(value: (Token, User)) -> Self {
        Self {
            token: value.0,
            user: value.1,
        }
    }
}

impl AuthService {
    pub fn new(p: &PgPool) -> DomainResult<Arc<Self>> {
        Ok(Arc::new(Self {
            repo: UserRepo::new(p),
            jwt: JwtService::new()?,
        }))
    }

    /// Creates a new user
    pub async fn signup(
        &self,
        p: SignupPayload,
    ) -> DomainResult<SignupResponse> {
        let u: User = self.repo.insert_user(p.into()).await?.into();
        let token = self.jwt.generate_token(u.id, u.username.as_ref())?;

        Ok((token, u).into())
    }

    /// Compares the given password with the one stored in the db
    ///
    /// Issues a JWT on success
    pub async fn login(&self, p: LoginPayload) -> DomainResult<()> {
        todo!()
    }
}

impl Into<SignupDb> for SignupPayload {
    fn into(self) -> SignupDb {
        todo!()
    }
}
