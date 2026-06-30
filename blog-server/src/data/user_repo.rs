use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::infra::DbError;

pub struct UserRepo(PgPool);

impl AsRef<PgPool> for UserRepo {
    fn as_ref(&self) -> &PgPool {
        &self.0
    }
}

pub struct UserDb {
    id: i64,
    username: String,
    email: String,
    password_hash: String,
    created_at: DateTime<Utc>,
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

type AuthResult = Result<UserDb, DbError>;

impl UserRepo {
    /// Reads a user
    pub async fn read_user(&self, id: i64) -> AuthResult {
        sqlx::query_as!(
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
        .map_err(Into::into)
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
}
