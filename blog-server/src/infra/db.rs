use sqlx::{PgPool, postgres::PgPoolOptions};
use thiserror::Error;
pub struct Database(PgPool);
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

#[derive(Debug, Error)]
pub enum DbError {
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("resource not found")]
    NotFound,
}
impl Database {
    pub fn new(url: &str) -> Result<Self, DbError> {
        Ok(Self(
            PgPoolOptions::new().max_connections(5).connect_lazy(url)?,
        ))
    }

    pub async fn migrate(&self) -> Result<(), DbError> {
        MIGRATOR.run(&self.0).await.map_err(Into::into)
    }
}
