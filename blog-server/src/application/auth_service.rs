use std::sync::Arc;

use crate::{
    data::UserRepo,
    domain::{
        DomainError, DomainResult, LoginPayload, Password, SignupPayload, User,
    },
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

    pub fn user(&self) -> &User {
        &self.user
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
        let db_user = match self.repo.insert_user(p.try_into()?).await {
            Ok(db_user) => db_user,
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
                return Err(match e.constraint() {
                    Some("users_username_key") => {
                        DomainError::UsernameAlreadyExists
                    }
                    Some("users_email_key") => DomainError::EmailAlreadyExists,
                    _ => DomainError::Database(e.to_string()),
                });
            }
            Err(e) => return Err(e.into()),
        };
        let u: User = db_user.into();
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
        let u = match self.repo.read_for_auth((&p).into()).await {
            Ok(u) => u,
            // Collapse "no such user" into the same error as "wrong
            // password" so a caller can't use login to enumerate accounts.
            Err(sqlx::Error::RowNotFound) => {
                return Err(DomainError::InvalidCredentials);
            }
            Err(e) => return Err(e.into()),
        };
        let h = Password::new_hashed(&u.password_hash);

        if p.get_password().validate(&h).is_err() {
            tracing::warn!(user_id = u.id, "login failed: bad password");
            return Err(DomainError::InvalidCredentials);
        }

        tracing::info!(user_id = u.id, "user logged in");
        Ok((self.jwt.generate_token(u.id, &u.username)?, u.into()).into())
    }
}
