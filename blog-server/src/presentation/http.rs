use crate::{
    application::{AuthService, BlogService},
    domain::{DomainError, DomainResult},
    infra::{Claims, Database, Token},
};
use actix_web::{
    FromRequest, ResponseError,
    http::{StatusCode, header::AUTHORIZATION},
    web,
};
use sqlx::PgPool;
use std::{pin::Pin, sync::Arc};

impl ResponseError for DomainError {
    fn status_code(&self) -> StatusCode {
        match self {
            DomainError::UserNotFound | DomainError::PostNotFound => {
                StatusCode::NOT_FOUND
            }
            DomainError::UsernameAlreadyExists
            | DomainError::EmailAlreadyExists => StatusCode::CONFLICT,
            DomainError::InvalidCredentials | DomainError::Unauthorized => {
                StatusCode::UNAUTHORIZED
            }
            DomainError::Forbidden => StatusCode::FORBIDDEN,
            DomainError::TypeMismatch
            | DomainError::Parsing(_)
            | DomainError::Password(_) => StatusCode::BAD_REQUEST,
            DomainError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl FromRequest for Claims {
    type Error = DomainError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        // Pull everything we need out of `req` synchronously, since neither
        // the header lookup nor `verify_token` is actually async - we only
        // box a future because that's the shape `FromRequest` demands.
        let token = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .map(|t| Token::new(t.to_string()));

        let state = req.app_data::<web::Data<AppState>>().cloned();

        Box::pin(async move {
            let token = token.ok_or(DomainError::Unauthorized)?;
            let state = state.ok_or(DomainError::Unauthorized)?;

            Ok(state.auth_service.jwt().verify_token(token)?)
        })
    }
}

/// The app state that all handlers share
pub struct AppState {
    auth_service: Arc<AuthService>,
    blog_service: Arc<BlogService>,
}

impl AppState {
    pub async fn new() -> DomainResult<Arc<Self>> {
        dotenvy::dotenv().ok();
        // to panic here is fine cause a misconfig toasts everything anyway
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let db = Database::new(&database_url)?;
        db.migrate().await?;

        Ok(Arc::new(Self {
            auth_service: AuthService::new(db.as_ref())?,
            blog_service: BlogService::new(db.as_ref()).await,
        }))
    }

    /// Builds state straight from an already-migrated pool, skipping env
    /// lookup and `migrate()`. Used by integration tests that get their
    /// pool from `#[sqlx::test]`, which migrates for them.
    pub async fn from_pool(pool: &PgPool) -> DomainResult<Arc<Self>> {
        Ok(Arc::new(Self {
            auth_service: AuthService::new(pool)?,
            blog_service: BlogService::new(pool).await,
        }))
    }

    /// Lets sibling modules (e.g. the auth middleware) reach the
    /// JwtService without needing direct access to `auth_service`.
    pub fn jwt(&self) -> &Arc<crate::infra::JwtService> {
        self.auth_service.jwt()
    }

    /// Lets the gRPC server share the same `AuthService`/`BlogService`
    /// instances as HTTP, instead of opening a second DB connection pool.
    pub fn auth_service(&self) -> Arc<AuthService> {
        self.auth_service.clone()
    }

    pub fn blog_service(&self) -> Arc<BlogService> {
        self.blog_service.clone()
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
        application::Pagination,
        domain::{DomainResult, Limit, Offset, Post, PostUpsert},
        infra::Claims,
        presentation::http::AppState,
    };
    use actix_web::{
        HttpResponse, delete, get, post, put,
        web::{self, Json},
    };
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct ListQuery {
        limit: Option<i64>,
        offset: Option<i64>,
    }

    #[get("/{id}")]
    pub async fn get_post(
        state: web::Data<AppState>,
        path: IdPath,
    ) -> DomainResult<Json<Post>> {
        Ok(Json(state.blog_service.get(path.into_inner()).await?))
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
        query: web::Query<ListQuery>,
    ) -> DomainResult<Json<Vec<Post>>> {
        let query = query.into_inner();
        Ok(Json(
            state
                .blog_service
                .list(Pagination::new(
                    query.offset.map(Offset::new),
                    query.limit.map(Limit::new),
                ))
                .await?,
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
