use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;
use blog_server::presentation::{
    http::{
        AppState,
        auth::{login, register},
        posts::{create_post, delete_post, get_post, list_posts, update_post},
    },
    middleware::jwt_validator,
};
use sqlx::PgPool;
use std::sync::{Arc, Once};

static JWT_SECRET_INIT: Once = Once::new();

/// `AuthService::new` reads `JWT_SECRET` via `JwtService::new`. Tests never
/// run `main`'s `dotenvy::dotenv()`, so a fixed, valid-base64 secret is set
/// once per test binary.
fn ensure_jwt_secret() {
    JWT_SECRET_INIT.call_once(|| unsafe {
        std::env::set_var(
            "JWT_SECRET",
            "dGVzdC1zZWNyZXQtZm9yLWFjdGl4LXdlYi10ZXN0cy1vbmx5",
        );
    });
}

/// Mirrors the route tree wired up in `main.rs`, minus CORS/logging.
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(web::scope("/auth").service(register).service(login))
            .service(
                web::scope("/posts")
                    .service(list_posts)
                    .service(get_post)
                    .service(
                        web::scope("")
                            .wrap(HttpAuthentication::bearer(jwt_validator))
                            .service(create_post)
                            .service(update_post)
                            .service(delete_post),
                    ),
            ),
    );
}

pub async fn app_state(pool: &PgPool) -> Arc<AppState> {
    ensure_jwt_secret();
    AppState::from_pool(pool)
        .await
        .expect("failed to build test AppState")
}
