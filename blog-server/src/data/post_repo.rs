use std::ops::Deref;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::{Limit, Offset};

/// The db version of the post
pub struct PostDb {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// The DB version of the update post
pub struct PostUpsertDb {
    pub author_id: i64,
    pub title: String,
    pub content: String,
}

pub struct PostRepo(PgPool);

impl AsRef<PgPool> for PostRepo {
    fn as_ref(&self) -> &PgPool {
        &self.0
    }
}

impl PostRepo {
    pub fn new(p: &PgPool) -> Self {
        Self(p.clone())
    }
}

type PostResult = Result<PostDb, sqlx::Error>;

impl PostRepo {
    /// Creates a new post
    pub async fn insert_post(&self, p: &PostUpsertDb) -> PostResult {
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
    pub async fn delete_post(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
           DELETE FROM posts
           WHERE id = $1
           "#,
            id
        )
        .execute(self.as_ref())
        .await?;

        Ok(())
    }

    /// Updates a given post
    pub async fn update_post<'a>(&self, u: &PostUpsertDb) -> PostResult {
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
    pub async fn list_posts(
        &self,
        author_id: i64,
        limit: Option<Limit>,
        offset: Option<Offset>,
    ) -> Result<Vec<PostDb>, sqlx::Error> {
        sqlx::query_as!(
            PostDb,
            r#"
           SELECT * FROM posts
           WHERE author_id = $1
           ORDER BY created_at
           LIMIT $2 OFFSET $3
           "#,
            author_id,
            limit.unwrap_or_default().get(),
            offset.unwrap_or_default().get()
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
