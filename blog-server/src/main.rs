use actix_web::{App, HttpServer, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use blog_server::presentation::{
    http::{
        AppState,
        auth::{login, register},
        posts::{create_post, delete_post, get_post, list_posts, update_post},
    },
    middleware::jwt_validator,
};
use std::time::Duration;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = AppState::new()
        .await
        .expect("failed to initialize app state");
    HttpServer::new(move || {
        App::new().app_data(web::Data::new(state.clone())).service(
            web::scope("/api")
                .service(web::scope("/auth").service(register).service(login))
                .service(
                    web::scope("/posts")
                        // public: reachable without a bearer token
                        .service(list_posts)
                        .service(get_post)
                        // protected: bearer token required
                        .service(
                            web::scope("")
                                .wrap(HttpAuthentication::bearer(jwt_validator))
                                .service(create_post)
                                .service(update_post)
                                .service(delete_post),
                        ),
                ),
        )
    })
    .bind(("127.0.0.1", 8080))?
    .keep_alive(Duration::from_secs(80))
    .run()
    .await
}
