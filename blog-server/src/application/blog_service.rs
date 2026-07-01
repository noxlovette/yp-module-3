use std::sync::Arc;

use crate::{
    data::PostRepo,
    domain::{DomainResult, Post, PostUpsert},
};
use sqlx::PgPool;

pub struct BlogService(PostRepo);

impl AsRef<PostRepo> for BlogService {
    fn as_ref(&self) -> &PostRepo {
        &self.0
    }
}

impl BlogService {
    pub async fn new(p: &PgPool) -> Arc<Self> {
        Arc::new(Self(PostRepo::new(p)))
    }

    /// Lists every post owned by `author_id`.
    pub async fn list(&self, author_id: i64) -> DomainResult<Vec<Post>> {
        let posts = self.as_ref().list_posts(author_id).await?;
        Ok(posts.into_iter().map(Into::into).collect())
    }

    /// Fetches a single post, ensuring `user_id` is the owner.
    pub async fn get(&self, user_id: i64, id: i64) -> DomainResult<Post> {
        let post: Post = self.as_ref().get_post(id).await?.into();
        post.validate_ownership(user_id)?;
        Ok(post)
    }

    /// Deletes a post, validates ownership
    pub async fn delete(&self, user_id: i64, id: i64) -> DomainResult<()> {
        let post: Post = self.as_ref().get_post(id).await?.into();
        post.validate_ownership(user_id)?;
        self.as_ref().delete_post(id).await?;
        Ok(())
    }

    /// Deletes a post, validates ownership
    pub async fn create(
        &self,
        user_id: i64,
        p: PostUpsert,
    ) -> DomainResult<Post> {
        Ok(self.as_ref().insert_post(&p.into_db(user_id)).await?.into())
    }

    /// Updates a post, validates ownership
    pub async fn update(
        &self,
        user_id: i64,
        post_id: i64,
        p: PostUpsert,
    ) -> DomainResult<Post> {
        let post: Post = self.as_ref().get_post(post_id).await?.into();
        post.validate_ownership(user_id)?;

        Ok(self.as_ref().update_post(&p.into_db(user_id)).await?.into())
    }
}
