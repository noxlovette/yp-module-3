use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use blog_proto::blog_service_server::BlogServiceServer;
use blog_server::presentation::{
    grpc::BlogGrpcService,
    http::{
        AppState,
        auth::{login, register},
        posts::{create_post, delete_post, get_post, list_posts, update_post},
    },
    middleware::jwt_validator,
};
use std::time::Duration;
use tonic::transport::Server as GrpcServer;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    blog_server::infra::init_logging();

    let state = AppState::new()
        .await
        .expect("failed to initialize app state");

    let grpc_service =
        BlogGrpcService::new(state.auth_service(), state.blog_service());

    let http_server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://frontend:4000")
            .allowed_origin("http://localhost:4000")
            .allow_any_header()
            .max_age(3600)
            .allowed_methods(vec!["GET", "POST", "DELETE", "PUT", "OPTIONS"]);

        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(cors)
            .app_data(web::Data::new(state.clone()))
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/auth").service(register).service(login),
                    )
                    .service(
                        web::scope("/posts")
                            // public: reachable without a bearer token
                            .service(list_posts)
                            .service(get_post)
                            // protected: bearer token required
                            .service(
                                web::scope("")
                                    .wrap(HttpAuthentication::bearer(
                                        jwt_validator,
                                    ))
                                    .service(create_post)
                                    .service(update_post)
                                    .service(delete_post),
                            ),
                    ),
            )
    })
    .bind(("0.0.0.0", 3000))?
    .keep_alive(Duration::from_secs(80))
    .run();

    let grpc_server = GrpcServer::builder()
        .add_service(BlogServiceServer::new(grpc_service))
        .serve("0.0.0.0:50051".parse()?);

    tracing::info!("http server listening on 0.0.0.0:3000");
    tracing::info!("grpc server listening on 0.0.0.0:50051");

    tokio::select! {
        res = http_server => res.map_err(anyhow::Error::from)?,
        res = grpc_server => res.map_err(anyhow::Error::from)?,
    }

    Ok(())
}
