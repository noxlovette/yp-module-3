# blog-server

Бэкенд: blog API на Postgres, доступный одновременно по HTTP (Actix Web) и gRPC (Tonic), с общим ядром бизнес-логики.

## Архитектура

Слои, примерно в порядке зависимостей:

```
presentation/   HTTP-хендлеры (Actix) + gRPC-сервис (Tonic) — тонкий, только транспорт
application/    AuthService, BlogService — оркестрация сценариев использования
domain/         сущности (User, Post), объекты-значения, DomainError
data/           sqlx-репозитории (UserRepo, PostRepo)
infra/          Database (пул соединений + миграции), JwtService, логирование
```

`presentation/http.rs` и `presentation/grpc.rs` — это тонкие обёртки вокруг одних и тех же экземпляров `AuthService`/`BlogService` (см. `AppState` в `presentation/http.rs`), поэтому бизнес-логика и валидация живут в одном месте независимо от того, через какой транспорт пришёл запрос.

Ошибки идут по тому же пути: код репозиториев/домена возвращает `DomainError`, который реализует `actix_web::ResponseError` (маппинг на HTTP-статусы — в `presentation/http.rs`) и конвертируется в `tonic::Status` для gRPC (`presentation/grpc.rs`).

## Запуск

Нужен доступный по `DATABASE_URL` Postgres; миграции применяются автоматически при старте (`sqlx::migrate!` в `infra/db.rs`).

```bash
cp ../.env.example .env   # укажите DATABASE_URL и JWT_SECRET
cargo run -p blog-server
```

Либо через `docker compose` из корня репозитория:

```bash
docker compose up --build
```

### Переменные окружения

| Переменная     | Обязательна | Комментарий                                                                 |
|----------------|:--:|------------------------------------------------------------------------------|
| `DATABASE_URL` | да | Строка подключения к Postgres, например `postgres://user:pass@host:5432/db` |
| `JWT_SECRET`   | да | Секрет в base64 для подписи/проверки HS512 JWT. Сгенерировать можно так:      |

```bash
openssl rand -base64 64
```

### Порты

- `8080` — HTTP (`/api/auth/*`, `/api/posts/*`)
- `50051` — gRPC (`blog.v1.BlogService`)

Оба сервера работают одновременно в одном процессе (`tokio::select!` в `main.rs`) и используют общее состояние.

## HTTP API

| Метод  | Путь                  | Авторизация | Описание |
|--------|-----------------------|:--:|-----------|
| POST   | `/api/auth/register`  | нет | Регистрация, возвращает токен |
| POST   | `/api/auth/login`     | нет | Возвращает токен |
| GET    | `/api/posts/`          | нет | Список постов (`?limit=&offset=`) |
| GET    | `/api/posts/{id}`      | нет | Получить один пост |
| POST   | `/api/posts/`          | да | Создать пост |
| PUT    | `/api/posts/{id}`      | да | Обновить пост (только автор) |
| DELETE | `/api/posts/{id}`      | да | Удалить пост (только автор) |

Защищённые маршруты ожидают заголовок `Authorization: Bearer <token>`. gRPC-сервис предоставляет те же операции в виде RPC (см. `blog.proto` в `blog-proto`), авторизация — так же через метаданные запроса.

## Авторизация и пароли

- Пароли хешируются через Argon2 (крейт `argon2`, `application/auth_service.rs`).
- JWT — HS512, подписываются `JWT_SECRET`, живут 24 часа (`infra/jwt.rs`).
- Проверка владения (update/delete) сравнивает `user_id` из JWT с `author_id` поста; при несовпадении возвращается `DomainError::Forbidden`.

## Тесты

Интеграционные тесты (`tests/`) поднимают настоящий Postgres через `#[sqlx::test]` и напрямую проверяют HTTP-хендлеры (`http_auth.rs`, `http_posts.rs`) и репозитории (`repo_user.rs`, `repo_post.rs`) — без моков базы данных.

```bash
cargo test -p blog-server
```

## Миграции

SQL-миграции лежат в `migrations/` и применяются автоматически при старте сервера через `sqlx::migrate!()`. Чтобы добавить новую, используйте `sqlx migrate add <name>` (sqlx-cli) из этой директории.
