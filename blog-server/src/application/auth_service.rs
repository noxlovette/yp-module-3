use std::sync::Arc;

use crate::{
    data::UserRepo,
    domain::{DomainResult, LoginPayload, Password, SignupPayload, User},
    infra::{JwtService, Token},
};
use serde::Serialize;
use sqlx::PgPool;

pub struct AuthService {
    repo: UserRepo,
    jwt: Arc<JwtService>,
}

#[derive(Serialize, Debug)]
pub struct UserToken {
    token: Token,
    user: User,
}

impl From<(Token, User)> for UserToken {
    fn from(value: (Token, User)) -> Self {
        Self {
            token: value.0,
            user: value.1,
        }
    }
}

impl UserToken {
    pub fn token(&self) -> &Token {
        &self.token
    }
}

impl AuthService {
    pub fn new(p: &PgPool) -> DomainResult<Arc<Self>> {
        Ok(Arc::new(Self {
            repo: UserRepo::new(p),
            jwt: JwtService::new()?,
        }))
    }

    /// Exposes the JwtService so the `Claims` extractor can verify
    /// incoming tokens without needing its own copy of the secret.
    pub fn jwt(&self) -> &Arc<JwtService> {
        &self.jwt
    }

    /// Creates a new user
    pub async fn signup(&self, p: SignupPayload) -> DomainResult<UserToken> {
        let u: User = self.repo.insert_user(p.try_into()?).await?.into();
        tracing::info!(
            user_id = u.id,
            username = u.username.as_ref(),
            "user signed up"
        );

        Ok((self.jwt.generate_token(u.id, u.username.as_ref())?, u).into())
    }

    /// Compares the given password with the one stored in the db
    ///
    /// Issues a JWT on success
    pub async fn login(&self, p: LoginPayload) -> DomainResult<UserToken> {
        let u = self.repo.read_for_auth((&p).into()).await?;
        let h = Password::new_hashed(&u.password_hash);

        if let Err(e) = p.get_password().validate(&h) {
            tracing::warn!(user_id = u.id, "login failed: bad password");
            return Err(e.into());
        }

        tracing::info!(user_id = u.id, "user logged in");
        Ok((self.jwt.generate_token(u.id, &u.username)?, u.into()).into())
    }
}
