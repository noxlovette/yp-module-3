use actix_web::{App, HttpServer, web};
use blog_server::presentation::http::{
    AppState,
    auth::{login, register},
    posts::{create_post, delete_post, get_post, list_posts, update_post},
};
use std::time::Duration;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = AppState::new();
    HttpServer::new(move || {
        App::new().app_data(web::Data::new(state.clone())).service(
            web::scope("/api")
                .service(web::scope("/auth").service(register).service(login))
                .service(
                    web::scope("/posts")
                        .service(create_post)
                        .service(update_post)
                        .service(delete_post)
                        .service(list_posts)
                        .service(get_post),
                ),
        )
    })
    .bind(("127.0.0.1", 8080))?
    .keep_alive(Duration::from_secs(80))
    .run()
    .await
}
