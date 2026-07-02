# blog-cli

Клиент командной строки для `blog-server`, построенный на `blog-client`.

## Использование

```bash
# HTTP на http://0.0.0.0:8080 (по умолчанию)
cargo run -p blog-cli -- register --username alice --email alice@example.com --password hunter2
cargo run -p blog-cli -- login --username alice --password hunter2
cargo run -p blog-cli -- create --title "Hello" --content "World"
cargo run -p blog-cli -- list --limit 10 --offset 0
cargo run -p blog-cli -- get --id 1
cargo run -p blog-cli -- update --id 1 --title "New title"
cargo run -p blog-cli -- delete --id 1

# обратиться к другому HTTP-хосту
cargo run -p blog-cli -- --http http://my-host:8080 list

# использовать gRPC вместо HTTP (фиксированный адрес http://0.0.0.0:50051)
cargo run -p blog-cli -- --grpc list
```

Релизный бинарник собирается через `cargo build --release -p blog-cli`; результат — `target/release/blog-cli`.

## Команды

| Команда    | Нужна авторизация | Описание |
|------------|:--:|-----------|
| `register` | нет | Создать аккаунт и сохранить полученный токен |
| `login`    | нет | Авторизоваться и сохранить полученный токен |
| `create`   | да | Создать пост |
| `get`      | нет | Получить пост по id |
| `update`   | да | Обновить пост (незаданные поля сохраняют текущее значение) |
| `delete`   | да | Удалить пост |
| `list`     | нет | Список постов с пагинацией через `--limit`/`--offset` |

## Хранение токена

`register` и `login` записывают полученный JWT в файл `./.blog-token` в текущей директории. Все последующие команды, требующие авторизации, читают токен оттуда — поэтому запускайте CLI из одной и той же директории каждый раз: при смене каталога (`cd`) сессия не найдётся. Явного logout или проверки истечения срока действия нет: чтобы сбросить сессию, просто удалите файл или выполните `login` заново — он перезапишет токен.

## Вывод

- Таблицы (`comfy-table`) для команд `list` и `get`.
- Спиннер (`indicatif`) на время выполнения запроса.
- Ошибки выводятся через `miette` с цепочкой контекста поверх исходной причины (например, «update of post 1 failed» оборачивает настоящую ошибку транспорта/HTTP) и подсказкой выполнить `login`, если файл с токеном не найден.
