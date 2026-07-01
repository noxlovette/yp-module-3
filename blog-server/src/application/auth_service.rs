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
        let u = self.repo.insert_user(p.into()).await?;
        todo!()
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
