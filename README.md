# Общие компоненты Benitoo

Один crate `common`, отдельный Git submodule. Прикладные правила и NATS DTO
остаются у сервисов; этот репозиторий содержит только общие технические компоненты.

| Feature | API | Зависимости и назначение |
|---|---|---|
| Без features | Пустой фасад | Нет runtime, Testcontainers и protobuf codegen |
| `contracts` | `common::contracts` | Канонические protobuf, Rust gRPC/WS types; требует `protoc` |
| `postgres-wakeup` | `common::postgres_wakeup::PostgresWakeup` | SQLx/Tokio, LISTEN/NOTIFY для outbox; уведомление лишь повод перечитать БД |
| `test-support` | `common::test_support` | Testcontainers harness PostgreSQL, Redis, NATS, RustFS, SMTP, Elasticsearch; только dev-dependency |

`default = []`. Production и dev-dependencies включают features независимо.
Harness не зависит от приложений, не применяет их migrations и не содержит бизнес-сценариев.
Сценарии, в том числе проверка готовности PostgreSQL после init scripts, находятся
в соседнем `integration_tests/tests`; fixtures конкретных сервисов остаются рядом с тестами.

## Контракты

Единственный источник — `contracts/`; `build.rs` работает только с feature `contracts`.
Смена расположения не меняет package, поля, номера или RPC. Flutter генерируется
из того же каталога командами родительского проекта. NATS-only схемы удаляются
парными изменениями отправителя и получателя; общего набора NATS DTO здесь нет.

## PostgreSQL wakeup

Worker подписывается до первого drain; после уведомления, reconnect или срока повтора
повторно читает durable состояние. Редкая страховочная проверка покрывает потерю
уведомлений. Реплики сохраняют свои SQL leases/idempotency; wakeup не заменяет outbox.
Drop отменяет принадлежащий объекту listener task.

## Подключение и проверка

```toml
[dependencies]
common = { path = "../common", features = ["contracts", "postgres-wakeup"] }

[dev-dependencies]
common = { path = "../common", features = ["test-support"] }
```

Submodule регистрируется по URL `git@github.com:JMURv/common.git`; обновление родителя
должно фиксировать опубликованный commit. Локальный initial commit не означает push
или существование remote. Проверки родителя и статус публикации — в его NEXT.
Для отдельной разработки crate можно запускать `cargo check --no-default-features`,
затем `cargo clippy --all-features --all-targets -- -D warnings`.
