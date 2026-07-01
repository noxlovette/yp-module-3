mod common;

use actix_web::{
    App,
    http::{StatusCode, header},
    test, web,
};
use blog_server::data::{PostRepo, PostUpsertDb, SignupDb, UserRepo};
use serde_json::json;
use sqlx::PgPool;

async fn seed_user(pool: &PgPool, username: &str) -> i64 {
    UserRepo::new(pool)
        .insert_user(SignupDb {
            username: username.into(),
            email: format!("{username}@example.com"),
            password_hash: "hashed-placeholder".into(),
        })
        .await
        .unwrap()
        .id
}

async fn seed_post(pool: &PgPool, author_id: i64, title: &str) -> i64 {
    PostRepo::new(pool)
        .insert_post(&PostUpsertDb {
            author_id,
            title: title.into(),
            content: "body".into(),
        })
        .await
        .unwrap()
        .id
}

fn bearer(token: &str) -> (header::HeaderName, String) {
    (header::AUTHORIZATION, format!("Bearer {token}"))
}

#[sqlx::test]
async fn list_posts_requires_a_bearer_token(pool: PgPool) {
    let state = common::app_state(&pool).await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(state))
            .configure(common::configure_routes),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/posts/").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn list_posts_rejects_a_garbled_token(pool: PgPool) {
    let state = common::app_state(&pool).await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(state))
            .configure(common::configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/posts/")
        .insert_header(bearer("not-a-real-jwt"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn get_post_with_a_non_numeric_id_is_not_found(pool: PgPool) {
    let state = common::app_state(&pool).await;
    let author_id = seed_user(&pool, "alice").await;
    let token = state.jwt().generate_token(author_id, "alice").unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(state))
            .configure(common::configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/posts/not-an-id")
        .insert_header(bearer(token.as_str()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn get_post_owned_by_another_user_is_forbidden(pool: PgPool) {
    let state = common::app_state(&pool).await;
    let alice = seed_user(&pool, "alice").await;
    let bob = seed_user(&pool, "bob").await;
    let bobs_post = seed_post(&pool, bob, "bob's private post").await;
    let alice_token = state.jwt().generate_token(alice, "alice").unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(state))
            .configure(common::configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/posts/{bobs_post}"))
        .insert_header(bearer(alice_token.as_str()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn create_post_persists_and_is_owned_by_the_caller(pool: PgPool) {
    let state = common::app_state(&pool).await;
    let alice = seed_user(&pool, "alice").await;
    let token = state.jwt().generate_token(alice, "alice").unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(state))
            .configure(common::configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/posts/")
        .insert_header(bearer(token.as_str()))
        .set_json(json!({ "title": "hello", "content": "world" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["author_id"], alice);
    assert_eq!(body["title"], "hello");
}

#[sqlx::test]
async fn create_post_without_a_token_is_unauthorized(pool: PgPool) {
    let state = common::app_state(&pool).await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(state))
            .configure(common::configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/posts/")
        .set_json(json!({ "title": "hello", "content": "world" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn create_post_rejects_a_malformed_body(pool: PgPool) {
    let state = common::app_state(&pool).await;
    let alice = seed_user(&pool, "alice").await;
    let token = state.jwt().generate_token(alice, "alice").unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(state))
            .configure(common::configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/posts/")
        .insert_header(bearer(token.as_str()))
        .set_json(json!({ "title": "missing content field" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn delete_post_then_get_is_not_found(pool: PgPool) {
    let state = common::app_state(&pool).await;
    let alice = seed_user(&pool, "alice").await;
    let post_id = seed_post(&pool, alice, "temp").await;
    let token = state.jwt().generate_token(alice, "alice").unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(state))
            .configure(common::configure_routes),
    )
    .await;

    let del = test::TestRequest::delete()
        .uri(&format!("/api/posts/{post_id}"))
        .insert_header(bearer(token.as_str()))
        .to_request();
    assert_eq!(
        test::call_service(&app, del).await.status(),
        StatusCode::NO_CONTENT
    );

    let get = test::TestRequest::get()
        .uri(&format!("/api/posts/{post_id}"))
        .insert_header(bearer(token.as_str()))
        .to_request();
    assert_eq!(
        test::call_service(&app, get).await.status(),
        StatusCode::NOT_FOUND
    );
}
