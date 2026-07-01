use blog_server::{
    data::{PostRepo, PostUpsertDb, SignupDb, UserRepo},
    domain::{Limit, Offset},
};
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

fn upsert(author_id: i64, title: &str, content: &str) -> PostUpsertDb {
    PostUpsertDb {
        author_id,
        title: title.into(),
        content: content.into(),
    }
}

#[sqlx::test]
async fn insert_then_get_post(pool: PgPool) {
    let author_id = seed_user(&pool, "alice").await;
    let repo = PostRepo::new(&pool);

    let created = repo
        .insert_post(&upsert(author_id, "hello", "world"))
        .await
        .unwrap();

    let fetched = repo.get_post(created.id).await.unwrap();
    assert_eq!(fetched.title, "hello");
    assert_eq!(fetched.author_id, author_id);
}

#[sqlx::test]
async fn list_posts_cap(pool: PgPool) {
    let alice = seed_user(&pool, "alice").await;
    let bob = seed_user(&pool, "bob").await;
    let repo = PostRepo::new(&pool);

    for i in 0..3 {
        repo.insert_post(&upsert(alice, &format!("a{i}"), "x"))
            .await
            .unwrap();
    }
    repo.insert_post(&upsert(bob, "b0", "x")).await.unwrap();

    let page = repo
        .list_posts(Limit::new(2), Offset::new(0))
        .await
        .unwrap();
    assert_eq!(page.len(), 2, "limit=2 should cap the page size");
}

#[sqlx::test]
async fn delete_post_removes_it(pool: PgPool) {
    let author_id = seed_user(&pool, "alice").await;
    let repo = PostRepo::new(&pool);
    let created = repo
        .insert_post(&upsert(author_id, "t", "c"))
        .await
        .unwrap();

    repo.delete_post(created.id).await.unwrap();

    let err = repo.get_post(created.id).await.err().unwrap();
    assert!(matches!(err, sqlx::Error::RowNotFound));
}

#[sqlx::test]
async fn update_post_only_touches_the_targeted_row(pool: PgPool) {
    let alice = seed_user(&pool, "alice").await;
    let bob = seed_user(&pool, "bob").await;
    let repo = PostRepo::new(&pool);

    let alice_post = repo
        .insert_post(&upsert(alice, "alice-original", "x"))
        .await
        .unwrap();
    let bob_post = repo
        .insert_post(&upsert(bob, "bob-original", "y"))
        .await
        .unwrap();

    repo.update_post(alice_post.id, &upsert(alice, "alice-updated", "z"))
        .await
        .unwrap();

    let alice_after = repo.get_post(alice_post.id).await.unwrap();
    assert_eq!(alice_after.title, "alice-updated");

    let bob_after = repo.get_post(bob_post.id).await.unwrap();
    assert_eq!(
        bob_after.title, "bob-original",
        "updating alice's post must not touch bob's post"
    );
}
