use sqlx::{PgPool, postgres::PgPoolOptions};
pub struct Database(PgPool);
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

impl Database {
    pub fn new(url: &str) -> Result<Self, sqlx::Error> {
        Ok(Self(
            PgPoolOptions::new().max_connections(5).connect_lazy(url)?,
        ))
    }

    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        MIGRATOR.run(&self.0).await.map_err(Into::into)
    }
}

impl AsRef<PgPool> for Database {
    fn as_ref(&self) -> &PgPool {
        &self.0
    }
}
