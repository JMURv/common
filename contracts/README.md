# Канонические protobuf-контракты

Единственный источник схем — этот каталог. Rust использует `common::contracts` с feature `contracts`,
Flutter — сгенерированные типы и клиенты из тех же схем. Строки SQL, секреты
провайдеров и внутренние адреса хранилища не становятся публичным контрактом.

## Границы

- `public/v1` — клиентский gRPC/gRPC-Web; `realtime/v1` — бинарный клиентский WebSocket.
- Внутренний gRPC сохраняется для операций чтения, снимков и материализации.
  Байты объектов передаются отдельно через подписанные S3 URL.
- Realtime-команды NATS сохраняют Protobuf и общие типы по DEC-44;
  `command.proto` импортирует только команды клиента. Остальные изменённые потоки
  используют DTO владельцев. При смене формата обе стороны и сохранённые сообщения
  меняются вместе; для старых сообщений согласуются опустошение очереди либо сброс dev.
- Формат клиентского WebSocket не заменяется JSON. Готовый кадр передаётся как bytes.

## Удаляемый контур

По DEC-01/17/22/23 исключены платформа Admin, корпоративная авторизация, отдельный SDK,
встроенные Meetings/code execution и платформа ботов. Обычные звонки, приглашения,
личные записи, WebAuthn, аналитика владельца и глобальный обзор администратора сохраняются.

Схемы Admin config/events, identity/directory/entitlements и analytics admin уже удалены.
После последних потребителей ServiceStatus удалены `benito/admin/v1/common.proto`,
`operations.proto`, экспорты Rust и проверки отсутствия этого RPC. Readiness не
зависит от protobuf; удалённые пакеты не возвращаются ради старого теста.

Code execution schema, её экспорты, четыре public RPC и realtime событие удалены
вместе с backend и Flutter consumers. Номер события 49 зарезервирован.
Calls/Invites/Recordings находятся в `public/v1/{calls,invites,recordings}.proto`.
Public Meetings schema и gateway domain удалены. Recordings RPC сохраняют свои
имена; запрос запуска содержит только room_id. Scope и meeting_id из записей,
три lifecycle события Meetings удалены; прежние номера полей зарезервированы.
Flutter consumers/bindings обновлены вместе с backend. Всего 38 схем; их вызовы
и импорты — в [карте контрактов](../../docs/architecture/contracts.md).
Внутренние Meeting RPC/metadata Messages и все bot schemas удалены; прежние
номера/имена полей зарезервированы. Общие realtime-команды сохраняются.

## Изменение и проверка

Сборочный скрипт собирает все `.proto` рекурсивно. При удалении схемы очищаются
импорты, экспорты Rust и её потребители. При смене клиентского контракта синхронно
обновляются обработчики backend, Flutter transports/bindings, команды генерации и тесты.

`task validate-all-contracts` компилирует общую библиотеку Rust.
`tools/ci/check_protocols.py` сравнивает полное свежее дерево Dart с сохранённым,
включая лишние файлы. Публичный API не получает второй бизнес-REST и обязательный RPC
проверки состояния worker.

Все AI потоки используют собственные JSON v2 DTO у Messages/gateway/AI worker.
AI namespace, job/transcription schemas и отдельные команды AI protobuf codegen удалены.
Их удалённые NATS-only схемы не возвращаются ради старого backlog JetStream.
Переход сохранённых сообщений описан в [карте событий](../../docs/architecture/events.md).

Fanout также использует собственный бинарный codec владельцев с готовым ServerFrame.
Его схема и специальная настройка Bytes codegen удалены. Realtime-команды
переиспользуют публичные типы Protobuf по DEC-44; `command.proto` сохраняется.
Публичный WS package разделён на девять предметных файлов с фасадом `realtime.proto`.

Открытая работа находится только в [СРОЧНЫХ](../../docs/planning/NEXT.md#срочные).
