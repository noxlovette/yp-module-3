use crate::{
    application::{ApplicationError, AuthError, AuthService, BlogService},
    infra::{Claims, DbError},
};
use actix_web::{FromRequest, ResponseError, web};
use std::{pin::Pin, sync::Arc};

// TODO: need to return correct response types and error messages
impl ResponseError for ApplicationError {}
impl ResponseError for AuthError {}
impl ResponseError for DbError {}

impl FromRequest for Claims {
    type Error = AuthError;
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
    use crate::{
        application::{ApplicationResult, PostReader},
        domain::{Post, PostUpsert},
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
    ) -> ApplicationResult<Json<Post>> {
        Ok(Json(
            state
                .blog_service
                .read(PostReader::single(
                    claims.get_user_id(),
                    path.into_inner(),
                ))?
                .try_into()?,
        ))
    }

    #[put("/{id}")]
    pub async fn update_post(
        state: web::Data<AppState>,
        path: IdPath,
        claims: Claims,
        payload: Json<PostUpsert>,
    ) -> ApplicationResult<Json<Post>> {
        Ok(Json(state.blog_service.update(
            claims.get_user_id(),
            path.into_inner(),
            payload.into_inner(),
        )?))
    }
    #[delete("/{id}")]
    pub async fn delete_post(
        state: web::Data<AppState>,
        claims: Claims,
        path: IdPath,
    ) -> ApplicationResult<HttpResponse> {
        state
            .blog_service
            .delete(claims.get_user_id(), path.into_inner())?;

        Ok(HttpResponse::NoContent().finish())
    }

    #[get("/")]
    pub async fn list_posts(
        state: web::Data<AppState>,
        claims: Claims,
    ) -> ApplicationResult<Json<Vec<Post>>> {
        Ok(Json(
            state
                .blog_service
                .read(PostReader::list(claims.get_user_id()))?
                .try_into()?,
        ))
    }

    #[post("/")]
    pub async fn create_post(
        state: web::Data<AppState>,
        claims: Claims,
        payload: Json<PostUpsert>,
    ) -> ApplicationResult<HttpResponse> {
        Ok(HttpResponse::Created().json(
            state
                .blog_service
                .create(claims.get_user_id(), payload.into_inner())?,
        ))
    }
}
