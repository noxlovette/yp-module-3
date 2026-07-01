use crate::domain::{Email, User, Username};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

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
    pub username: String,
    pub password_hash: String,
    pub email: String,
}

pub struct AuthUserDb {
    pub username: String,
    pub password_hash: String,
    pub id: i64,
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

pub enum LoginCaller<'a> {
    Email(&'a Email),
    Username(&'a Username),
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

    pub async fn read_for_auth<'a>(
        &self,
        reader: LoginCaller<'a>,
    ) -> Result<UserDb, sqlx::Error> {
        match reader {
            LoginCaller::Email(e) => {
                sqlx::query_as!(
                    UserDb,
                    "SELECT * from users where email = $1",
                    e.as_ref()
                )
                .fetch_one(self.as_ref())
                .await
            }
            LoginCaller::Username(u) => {
                sqlx::query_as!(
                    UserDb,
                    "SELECT * from users where username = $1",
                    u.as_ref()
                )
                .fetch_one(self.as_ref())
                .await
            }
        }
    }
}
