use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    data::{PostDb, PostUpsertDb},
    domain::{DomainError, DomainResult},
};

#[derive(Debug, Serialize)]
pub struct Post {
    id: i64,
    title: String,
    content: String,
    author_id: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct PostUpsert {
    pub title: String,
    pub content: String,
}

impl Post {
    /// Checks that `user_id` is allowed to read/write this post.
    pub fn validate_ownership(&self, user_id: i64) -> DomainResult<()> {
        if self.author_id != user_id {
            return Err(DomainError::Forbidden);
        }
        Ok(())
    }
}

impl From<PostDb> for Post {
    fn from(value: PostDb) -> Self {
        Post {
            id: value.id,
            title: value.title,
            content: value.content,
            author_id: value.author_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl PostUpsert {
    pub fn into_db(self, author_id: i64) -> PostUpsertDb {
        PostUpsertDb {
            author_id,
            title: self.title,
            content: self.content,
        }
    }
}
