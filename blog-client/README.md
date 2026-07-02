# blog-client

Rust-библиотека-клиент для `blog-server`, работающая либо по HTTP, либо по gRPC — под единым API. Используется в `blog-cli`; подходит для любого Rust-приложения, которому нужно обращаться к blog-сервису, не заботясь о том, какой транспорт используется на проводе.

## Использование

```rust
use blog_client::{BlogClient, Transport};

// либо Transport::Http(Some("http://my-host:8080".into())) для другого адреса,
// либо Transport::Grpc для gRPC (по умолчанию http://0.0.0.0:50051)
let mut client = BlogClient::new(Transport::Http(None)).await?;

let auth = client.login("alice".into(), "hunter2".into()).await?;
// login()/register() сами сохраняют полученный токен в клиенте;
// client.set_token(...) нужен только при восстановлении ранее сохранённого токена
let posts = client.list_posts(None, None).await?;
```

## Устройство

`BlogClient` оборачивает enum `ClientKind` (`Http(HttpClient)` / `Grpc(GrpcClient)`), который выбирается один раз при создании через `Transport`. Каждый публичный метод (`register`, `login`, `create_post`, ...) внутри делает `match` по транспорту и вызывает эквивалентную операцию — так вызывающий код остаётся независимым от транспорта, а выбор HTTP или gRPC делается один раз, в точке вызова `BlogClient::new`.

- **`grpc_client.rs`** — тонкий алиас типа поверх сгенерированного `tonic`-ом `BlogServiceClient<Channel>` из `blog-proto`; самое интересное — функция `with_auth` в `lib.rs`, которая добавляет `authorization: Bearer <token>` в метаданные исходящего gRPC-запроса (в отличие от `reqwest` с его `.bearer_auth()`, у gRPC нет готового метода для этого).
- **`http_client.rs`** — клиент на `reqwest`, обращающийся к маршрутам `/api/*` у `blog-server`. Обратите внимание на `LoginPayloadDto`: у `blog-server` payload для логина — это внешне тегированный enum (`{"Username": {...}}`), поэтому здесь есть отдельный клиентский DTO под эту форму, отличный от сгенерированного из proto `LoginRequest`, который используется всюду ещё.
- **`error.rs`** — `BlogClientError` объединяет ошибки обоих транспортов (`reqwest::Error`, `tonic::Status`, `tonic::transport::Error`) в один enum, маппя HTTP-коды и gRPC-статусы на одни и те же варианты (`NotFound`, `Unauthorized`, `Forbidden`, ...), чтобы вызывающему коду не приходилось обрабатывать ошибки по-разному в зависимости от транспорта.
- **`Limit`/`Offset`** — newtype-обёртки для пагинации, гарантирующие неотрицательность (`TryFrom<i64>`), используются в `list_posts`.

## Авторизация

Клиент хранит состояние авторизации: `register`/`login` сохраняют полученный токен внутри клиента, и последующие вызовы защищённых методов (`create_post`, `update_post`, `delete_post`) читают его через `require_token()`, возвращая `BlogClientError::Unauthorized`, если токен не задан. Используйте `set_token`/`get_token`, чтобы сохранять/восстанавливать токен между экземплярами клиента (пример — файловое хранилище `TokenStorage` в `blog-cli`).
