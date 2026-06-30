use crate::data::PostRepo;
use sqlx::PgPool;

pub struct BlogService(PostRepo);

impl BlogService {
    pub fn new(p: &PgPool) -> Self {
        Self(PostRepo::new(p))
    }
}
