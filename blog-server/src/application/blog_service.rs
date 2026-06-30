use std::{io::Read, sync::Arc};

use crate::{
    application::{ApplicationError, ApplicationResult},
    data::PostRepo,
    domain::{Post, PostUpsert},
};
use sqlx::PgPool;

pub struct BlogService(PostRepo);

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

impl From<ReadOut> for ApplicationError {
    fn from(_: ReadOut) -> Self {
        ApplicationError::TypeMismatch
    }
}
impl BlogService {
    pub fn new(p: &PgPool) -> Arc<Self> {
        Arc::new(Self(PostRepo::new(p)))
    }

    pub fn read(&self, reader: PostReader) -> ApplicationResult<ReadOut> {
        todo!("validate ownership")
    }

    pub fn delete(&self, user_id: i64, id: i64) -> ApplicationResult<()> {
        todo!("validate ownership")
    }

    pub fn create(
        &self,
        user_id: i64,
        p: PostUpsert,
    ) -> ApplicationResult<Post> {
        todo!()
    }

    pub fn update(
        &self,
        user_id: i64,
        post_id: i64,
        p: PostUpsert,
    ) -> ApplicationResult<Post> {
        todo!("validate ownership")
    }
}
