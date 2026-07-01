use crate::{
    data::PostRepo,
    domain::{DomainResult, Limit, Offset, Post, PostUpsert},
};
use sqlx::PgPool;
use std::sync::Arc;

pub struct BlogService(PostRepo);
pub struct Pagination {
    offset: Offset,
    limit: Limit,
}

impl Pagination {
    pub fn new(o: Option<Offset>, l: Option<Limit>) -> Self {
        Self {
            offset: o.unwrap_or_default(),
            limit: l.unwrap_or_default(),
        }
    }
}
impl AsRef<PostRepo> for BlogService {
    fn as_ref(&self) -> &PostRepo {
        &self.0
    }
}

impl BlogService {
    pub async fn new(p: &PgPool) -> Arc<Self> {
        Arc::new(Self(PostRepo::new(p)))
    }

    /// Lists every post available
    pub async fn list(&self, p: Pagination) -> DomainResult<Vec<Post>> {
        let posts = self.as_ref().list_posts(p.limit, p.offset).await?;
        Ok(posts.into_iter().map(Into::into).collect())
    }

    /// Fetches a single post
    pub async fn get(&self, id: i64) -> DomainResult<Post> {
        let post: Post = self.as_ref().get_post(id).await?.into();
        Ok(post)
    }

    /// Deletes a post, validates ownership
    pub async fn delete(&self, user_id: i64, id: i64) -> DomainResult<()> {
        let post: Post = self.as_ref().get_post(id).await?.into();
        post.validate_ownership(user_id)?;
        self.as_ref().delete_post(id).await?;
        tracing::info!(user_id, post_id = id, "post deleted");
        Ok(())
    }

    /// Deletes a post, validates ownership
    pub async fn create(
        &self,
        user_id: i64,
        p: PostUpsert,
    ) -> DomainResult<Post> {
        let post: Post =
            self.as_ref().insert_post(&p.into_db(user_id)).await?.into();
        tracing::info!(user_id, post_id = post.id(), "post created");
        Ok(post)
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

        let updated: Post = self
            .as_ref()
            .update_post(post_id, &p.into_db(user_id))
            .await?
            .into();
        tracing::info!(user_id, post_id, "post updated");
        Ok(updated)
    }
}
