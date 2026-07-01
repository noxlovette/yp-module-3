use blog_server::data::{ReaderCaller, SignupDb, UserRepo};
use sqlx::PgPool;

fn signup(username: &str, email: &str) -> SignupDb {
    SignupDb {
        username: username.into(),
        email: email.into(),
        password_hash: "hashed-placeholder".into(),
    }
}

#[sqlx::test]
async fn insert_then_read_by_id_and_username(pool: PgPool) {
    let repo = UserRepo::new(&pool);
    let created = repo
        .insert_user(signup("alice", "alice@example.com"))
        .await
        .unwrap();

    let by_id = repo.read_user(ReaderCaller::Id(created.id)).await.unwrap();
    assert_eq!(by_id.username, "alice");

    let by_username = repo
        .read_user(ReaderCaller::Username("alice".into()))
        .await
        .unwrap();
    assert_eq!(by_username.id, created.id);
}

#[sqlx::test]
async fn read_missing_user_is_row_not_found(pool: PgPool) {
    let repo = UserRepo::new(&pool);
    let err = repo.read_user(ReaderCaller::Id(999)).await.err().unwrap();
    assert!(matches!(err, sqlx::Error::RowNotFound));
}

#[sqlx::test]
async fn duplicate_username_violates_unique_constraint(pool: PgPool) {
    let repo = UserRepo::new(&pool);
    repo.insert_user(signup("bob", "bob@example.com"))
        .await
        .unwrap();

    let err = repo
        .insert_user(signup("bob", "bob2@example.com"))
        .await
        .err()
        .unwrap();

    let sqlx::Error::Database(db_err) = &err else {
        panic!("expected a database error, got {err:?}");
    };
    assert_eq!(db_err.code().as_deref(), Some("23505")); // unique_violation
}

#[sqlx::test]
async fn read_for_auth_finds_by_email_and_username(pool: PgPool) {
    use blog_server::{
        data::LoginCaller,
        domain::{Email, Username},
    };

    let repo = UserRepo::new(&pool);
    let created = repo
        .insert_user(signup("carol", "carol@example.com"))
        .await
        .unwrap();

    let email = Email::new("carol@example.com");
    let by_email = repo
        .read_for_auth(LoginCaller::Email(&email))
        .await
        .unwrap();
    assert_eq!(by_email.id, created.id);

    let username = Username::new("carol");
    let by_username = repo
        .read_for_auth(LoginCaller::Username(&username))
        .await
        .unwrap();
    assert_eq!(by_username.id, created.id);
}
