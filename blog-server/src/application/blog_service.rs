use std::sync::Arc;

use crate::{application::ApplicationResult, data::PostRepo, domain::Post};
use sqlx::PgPool;

pub struct BlogService(PostRepo);

pub enum PostReader {
    List { author_id: i64 },
    One { id: i64 },
}

pub enum ReadOut {
    List(Vec<Post>),
    One(Post),
}

impl BlogService {
    pub fn new(p: &PgPool) -> Arc<Self> {
        Arc::new(Self(PostRepo::new(p)))
    }

    pub fn read(&self, reader: PostReader) -> ApplicationResult<ReadOut> {
        todo!()
    }

    pub fn delete(&self, id: i64) -> ApplicationResult<()> {
        todo!()
    }

    pub fn create(&self, id: i64) -> ApplicationResult<Post> {
        todo!()
    }

    pub fn update(&self, id: i64) -> ApplicationResult<Post> {
        todo!()
    }
}
