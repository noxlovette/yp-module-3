use std::sync::Arc;

use crate::{
    data::PostRepo,
    domain::{DomainError, DomainResult, Post, PostUpsert},
};
use sqlx::PgPool;

pub struct BlogService(PostRepo);

impl AsRef<PostRepo> for BlogService {
    fn as_ref(&self) -> &PostRepo {
        &self.0
    }
}

pub enum PostReader {
    List { author_id: i64 },
    Single { author_id: i64, id: i64 },
}

impl PostReader {
    pub fn list(id: i64) -> Self {
        Self::List { author_id: id }
    }

    pub fn single(author_id: i64, id: i64) -> Self {
        Self::Single { author_id, id }
    }
}

pub enum ReadOut {
    List(Vec<Post>),
    One(Post),
}
impl TryFrom<ReadOut> for Vec<Post> {
    type Error = ReadOut;

    fn try_from(value: ReadOut) -> Result<Self, Self::Error> {
        match value {
            ReadOut::List(v) => Ok(v),
            other => Err(other),
        }
    }
}

impl TryFrom<ReadOut> for Post {
    type Error = ReadOut;

    fn try_from(value: ReadOut) -> Result<Self, Self::Error> {
        match value {
            ReadOut::One(p) => Ok(p),
            other => Err(other),
        }
    }
}

impl From<ReadOut> for DomainError {
    fn from(_: ReadOut) -> Self {
        DomainError::TypeMismatch
    }
}
impl BlogService {
    pub async fn new(p: &PgPool) -> Arc<Self> {
        Arc::new(Self(PostRepo::new(p)))
    }

    pub async fn read(&self, reader: PostReader) -> DomainResult<ReadOut> {
        todo!("validate ownership")
    }

    pub async fn delete(&self, user_id: i64, id: i64) -> DomainResult<()> {
        todo!("validate ownership")
    }

    pub async fn create(
        &self,
        user_id: i64,
        p: PostUpsert,
    ) -> DomainResult<Post> {
        todo!()
    }

    pub async fn update(
        &self,
        user_id: i64,
        post_id: i64,
        p: PostUpsert,
    ) -> DomainResult<Post> {
        todo!("validate ownership")
    }
}
