use crate::application::{
    ApplicationError, AuthError, AuthService, BlogService,
};
use actix_web::{ResponseError, web};
use std::sync::Arc;

// TODO: both need to return correct response types and error messages
impl ResponseError for ApplicationError {}
impl ResponseError for AuthError {}

/// The app state that all handlers share
pub struct AppState {
    auth_service: Arc<AuthService>,
    blog_service: Arc<BlogService>,
}

type IdPath = web::Path<i64>;

pub mod auth {
    use crate::{
        application::ApplicationResult,
        domain::{LoginPayload, SignupPayload},
        presentation::http::AppState,
    };
    use actix_web::{
        HttpResponse, post,
        web::{self, Json},
    };

    #[post("/register")]
    pub async fn register(
        state: web::Data<AppState>,
        payload: Json<SignupPayload>,
    ) -> ApplicationResult<HttpResponse> {
        Ok(HttpResponse::Created()
            .json(state.auth_service.signup(payload.into_inner())?))
    }

    #[post("/login")]
    pub async fn login(
        state: web::Data<AppState>,
        payload: Json<LoginPayload>,
    ) -> ApplicationResult<HttpResponse> {
        Ok(HttpResponse::Ok()
            .json(state.auth_service.login(payload.into_inner())?))
    }
}
pub mod posts {
    use super::IdPath;
    use crate::domain::PostUpsert;
    use actix_web::{Responder, delete, get, post, put, web::Json};

    #[get("/{id}")]
    pub async fn get_post(path: IdPath) -> impl Responder {
        let id = path.into_inner();

        todo!("postser")
    }

    #[put("/{id}")]
    pub async fn update_post(
        path: IdPath,
        payload: Json<PostUpsert>,
    ) -> impl Responder {
        let id = path.into_inner();
    }
    #[delete("/{id}")]
    pub async fn delete_post(path: IdPath) -> impl Responder {
        let id = path.into_inner();
    }

    #[get("/")]
    pub async fn list_posts() -> impl Responder {
        todo!()
    }

    #[post("/")]
    pub async fn create_post(payload: Json<PostUpsert>) -> impl Responder {
        todo!()
    }
}
