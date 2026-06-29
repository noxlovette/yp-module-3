use crate::infra::DbError;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// The db version of the post
pub struct PostDb {
    id: i64,
    title: String,
    content: String,
    author_id: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// The DB version of the update post
pub struct PostUpsertDb<'a> {
    author_id: i64,
    title: &'a str,
    content: &'a str,
}

struct PostRepo(PgPool);

impl AsRef<PgPool> for PostRepo {
    fn as_ref(&self) -> &PgPool {
        &self.0
    }
}

type PostResult = Result<PostDb, DbError>;

impl PostRepo {
    /// Creates a new post
    pub async fn insert_post<'a>(&self, p: &PostUpsertDb<'a>) -> PostResult {
        sqlx::query_as!(
            PostDb,
            r#"
            INSERT INTO posts
            (
            author_id, title, content
            )
            VALUES
            (
            $1,
            $2,
            $3
            )
            RETURNING *;
            "#,
            p.author_id,
            p.title,
            p.content
        )
        .fetch_one(self.as_ref())
        .await
        .map_err(Into::into)
    }

    /// Deletes a given post
    ///
    /// Does not validate ownership
    pub async fn delete_post(&self, id: i64) -> Result<(), DbError> {
        let qr = sqlx::query!(
            r#"
           DELETE FROM posts
           WHERE id = $1
           "#,
            id
        )
        .execute(self.as_ref())
        .await?;

        if qr.rows_affected() == 0 {
            return Err(DbError::NotFound);
        }

        Ok(())
    }

    /// Updates a given post
    pub async fn update_post<'a>(&self, u: &PostUpsertDb<'a>) -> PostResult {
        sqlx::query_as!(
            PostDb,
            r#"
           UPDATE posts
           SET title = $1, content = $2
           RETURNING *;
           "#,
            u.title,
            u.content
        )
        .fetch_one(self.as_ref())
        .await
        .map_err(Into::into)
    }

    /// Gets all posts for given user
    pub async fn list_posts(&self, author_id: i64) -> Result<Vec<PostDb>, DbError> {
        sqlx::query_as!(
            PostDb,
            r#"
           SELECT * FROM posts
           WHERE author_id = $1
           "#,
            author_id
        )
        .fetch_all(self.as_ref())
        .await
        .map_err(Into::into)
    }

    /// Retrieves a post by its id
    ///
    /// Does not validate post ownership
    pub async fn get_post(&self, id: i64) -> PostResult {
        sqlx::query_as!(
            PostDb,
            r#"
            SELECT * from posts
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(self.as_ref())
        .await
        .map_err(Into::into)
    }
}
