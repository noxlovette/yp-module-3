mod common;

use actix_web::{App, http::StatusCode, test, web};
use blog_server::{
    data::{SignupDb, UserRepo},
    domain::Password,
};
use serde_json::json;
use sqlx::PgPool;

async fn seed_user(
    pool: &PgPool,
    username: &str,
    email: &str,
    plain_password: &str,
) -> i64 {
    let password_hash = Password::new_plain(plain_password)
        .hash()
        .unwrap()
        .get_hash()
        .unwrap();
    UserRepo::new(pool)
        .insert_user(SignupDb {
            username: username.into(),
            email: email.into(),
            password_hash,
        })
        .await
        .unwrap()
        .id
}

#[sqlx::test]
async fn register_returns_created_with_token_and_user(pool: PgPool) {
    let state = common::app_state(&pool).await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(state))
            .configure(common::configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "supersecret1",
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["token"].is_string());
    assert_eq!(body["user"]["username"], "alice");
}

#[sqlx::test]
async fn register_rejects_a_too_short_password(pool: PgPool) {
    let state = common::app_state(&pool).await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(state))
            .configure(common::configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(json!({ "username": "bob", "email": "bob@example.com", "password": "short" }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn register_rejects_a_duplicate_username(pool: PgPool) {
    seed_user(&pool, "carol", "carol@example.com", "carolpassword").await;
    let state = common::app_state(&pool).await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(state))
            .configure(common::configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(json!({ "username": "carol", "email": "carol2@example.com", "password": "anotherpassword" }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::CONFLICT,
        "duplicate username should be reported as a conflict, not a generic \
         server error"
    );
}

#[sqlx::test]
async fn login_with_correct_password_returns_ok(pool: PgPool) {
    seed_user(&pool, "dave", "dave@example.com", "davepassword1").await;
    let state = common::app_state(&pool).await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(state))
            .configure(common::configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({ "Username": { "username": "dave", "password": "davepassword1" } }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test]
async fn login_with_wrong_password_is_unauthorized(pool: PgPool) {
    seed_user(&pool, "erin", "erin@example.com", "erinpassword1").await;
    let state = common::app_state(&pool).await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(state))
            .configure(common::configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({ "Username": { "username": "erin", "password": "wrongpassword" } }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn login_with_unknown_username_is_unauthorized(pool: PgPool) {
    let state = common::app_state(&pool).await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(state))
            .configure(common::configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({ "Username": { "username": "ghost", "password": "whatever1" } }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
