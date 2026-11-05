# Общие компоненты Benitoo

Один crate `common`, отдельный Git submodule. Прикладные правила и DTO событий
остаются у сервисов. Исключение — согласованные общие Protobuf-команды realtime:
они переиспользуют публичные типы, а не копируют их в каждый сервис (DEC-44 родителя).

| Feature | API | Зависимости и назначение |
|---|---|---|
| Без features | Пустой фасад | Нет runtime, Testcontainers и protobuf codegen |
| `contracts` | `common::contracts` | Канонические protobuf, Rust gRPC/WS types; требует `protoc` |
| `realtime` | Методы общих Protobuf-типов | Включает `contracts`; конструктор NATS command envelope, проекции входных вложений и poll. UUID/serde_json; без авторизации и I/O |
| `postgres-wakeup` | `common::postgres_wakeup::PostgresWakeup` | SQLx/Tokio, LISTEN/NOTIFY для outbox; уведомление лишь повод перечитать БД |
| `test-support` | `common::test_support` | Testcontainers harness PostgreSQL, Redis, NATS, RustFS, SMTP, Elasticsearch; только dev-dependency |

`default = []`. Production и dev-dependencies включают features независимо.
Harness не зависит от приложений, не применяет их migrations и не содержит бизнес-сценариев.
Сценарии, в том числе проверка готовности PostgreSQL после init scripts, находятся
в соседнем `integration_tests/tests`; fixtures конкретных сервисов остаются рядом с тестами.

## Контракты

Единственный источник — `contracts/`; `build.rs` работает только с feature `contracts`.
Смена расположения не меняет package, поля, номера или RPC. Flutter генерируется
из того же каталога командами родительского проекта. AI JSON и бинарный fanout
принадлежат сервисам; `realtime/internal/v1/command.proto` сохраняется для NATS.

Публичный package `benito.realtime.v1` разделён на `commands`, `message_types`,
`messages`, `ai`, `recordings`, `presence`, `communities`, `moderation`.
`realtime.proto` содержит кадры и служебные ответы, экспортирует эти схемы через
`import public`. Rust namespace и Flutter import `realtime.pb.dart` сохраняются.
Для Dart генерируются все девять файлов, а не только фасад.

Feature `realtime` добавляет методы к каноническим типам: `RealtimeCommandEnvelope::new`
и `MediaAttachment::into_command_value` / `MessageMetadata::into_command_value`.
Проекция вложения сохраняет входную revision, не переносит выходные read URL.
Модель не копируется; сервисы сохраняют свои контекстные проверки и ошибки.

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
