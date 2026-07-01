use crate::{
    application::{AuthService, BlogService},
    domain::DomainError,
    infra::Claims,
};
use actix_web::{FromRequest, ResponseError, web};
use std::{pin::Pin, sync::Arc};

// TODO: need to return correct response types and error messages
impl ResponseError for DomainError {}

impl FromRequest for Claims {
    type Error = DomainError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn extract(req: &actix_web::HttpRequest) -> Self::Future {
        todo!("get the token from the meow")
    }

    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        todo!(
            "validate exp, validate the token, get the jwtservice from appstate"
        )
    }
}

/// The app state that all handlers share
pub struct AppState {
    auth_service: Arc<AuthService>,
    blog_service: Arc<BlogService>,
}

impl AppState {
    pub fn new() -> Arc<Self> {
        todo!()
    }
}

type IdPath = web::Path<i64>;

pub mod auth {
    use crate::{
        domain::{DomainResult, LoginPayload, SignupPayload},
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
    ) -> DomainResult<HttpResponse> {
        Ok(HttpResponse::Created()
            .json(state.auth_service.signup(payload.into_inner()).await?))
    }

    #[post("/login")]
    pub async fn login(
        state: web::Data<AppState>,
        payload: Json<LoginPayload>,
    ) -> DomainResult<HttpResponse> {
        Ok(HttpResponse::Ok()
            .json(state.auth_service.login(payload.into_inner()).await?))
    }
}

pub mod posts {
    use super::IdPath;
    use crate::{
        application::PostReader,
        domain::{DomainResult, Post, PostUpsert},
        infra::Claims,
        presentation::http::AppState,
    };
    use actix_web::{
        HttpResponse, delete, get, post, put,
        web::{self, Json},
    };

    #[get("/{id}")]
    pub async fn get_post(
        state: web::Data<AppState>,
        claims: Claims,
        path: IdPath,
    ) -> DomainResult<Json<Post>> {
        Ok(Json(
            state
                .blog_service
                .read(PostReader::single(
                    claims.get_user_id(),
                    path.into_inner(),
                ))
                .await?
                .try_into()?,
        ))
    }

    #[put("/{id}")]
    pub async fn update_post(
        state: web::Data<AppState>,
        path: IdPath,
        claims: Claims,
        payload: Json<PostUpsert>,
    ) -> DomainResult<Json<Post>> {
        Ok(Json(
            state
                .blog_service
                .update(
                    claims.get_user_id(),
                    path.into_inner(),
                    payload.into_inner(),
                )
                .await?,
        ))
    }
    #[delete("/{id}")]
    pub async fn delete_post(
        state: web::Data<AppState>,
        claims: Claims,
        path: IdPath,
    ) -> DomainResult<HttpResponse> {
        state
            .blog_service
            .delete(claims.get_user_id(), path.into_inner())
            .await?;

        Ok(HttpResponse::NoContent().finish())
    }

    #[get("/")]
    pub async fn list_posts(
        state: web::Data<AppState>,
        claims: Claims,
    ) -> DomainResult<Json<Vec<Post>>> {
        Ok(Json(
            state
                .blog_service
                .read(PostReader::list(claims.get_user_id()))
                .await?
                .try_into()?,
        ))
    }

    #[post("/")]
    pub async fn create_post(
        state: web::Data<AppState>,
        claims: Claims,
        payload: Json<PostUpsert>,
    ) -> DomainResult<HttpResponse> {
        Ok(HttpResponse::Created().json(
            state
                .blog_service
                .create(claims.get_user_id(), payload.into_inner())
                .await?,
        ))
    }
}
