use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::Username;

pub struct UserRepo(PgPool);

impl AsRef<PgPool> for UserRepo {
    fn as_ref(&self) -> &PgPool {
        &self.0
    }
}

pub struct UserDb {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

pub struct SignupDb {
    username: String,
    password_hash: String,
    email: String,
}

impl UserRepo {
    pub fn new(p: &PgPool) -> Self {
        Self(p.clone())
    }
}

type AuthResult = Result<UserDb, sqlx::Error>;

pub enum ReaderCaller {
    Id(i64),
    Username(Username),
}

impl UserRepo {
    /// Reads a user either by id or username
    pub async fn read_user(&self, reader: ReaderCaller) -> AuthResult {
        use ReaderCaller::*;
        match reader {
            Id(id) => sqlx::query_as!(
                UserDb,
                r#"
            SELECT id, username, email, password_hash, created_at
            FROM users
            WHERE id = $1
        "#,
                id
            )
            .fetch_one(self.as_ref())
            .await
            .map_err(Into::into),
            Username(username) => sqlx::query_as!(
                UserDb,
                r#"
        SELECT id, username, email, password_hash, created_at
        FROM users
        WHERE username = $1
    "#,
                username.as_ref()
            )
            .fetch_one(self.as_ref())
            .await
            .map_err(Into::into),
        }
    }

    /// Creates a new user
    pub async fn insert_user(&self, i: SignupDb) -> AuthResult {
        sqlx::query_as!(
            UserDb,
            r#"
            INSERT INTO users
            (
           username,
           email,
           password_hash
            )
            VALUES
            (
            $1,
            $2,
            $3
            )
            RETURNING *;
            "#,
            i.username,
            i.email,
            i.password_hash
        )
        .fetch_one(self.as_ref())
        .await
        .map_err(Into::into)
    }

    pub async fn read_password_hash(
        &self,
        id: i64,
    ) -> Result<String, sqlx::Error> {
        sqlx::query_scalar!("SELECT password_hash from users where id = $1", id)
            .fetch_one(self.as_ref())
            .await
    }
}
