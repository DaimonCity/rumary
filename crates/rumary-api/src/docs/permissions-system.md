# Система прав (Permissions) в стиле LuckPerms

Документ описывает то, что реализовано, а не план. Логика — в крейте
`rumary-perms`, доменные типы — в `rumary-dto` (`domain::perms`), реализация
поверх Postgres — в `rumary-api/src/repo/perms.rs`, точка входа для хендлеров —
`rumary-api/src/service/permissions.rs`.

## 1. Что взято из LuckPerms

- **Permission node** — ключ вида `configuration.get`, `configuration.*`, `*`.
  Wildcard разрешён только как целый сегмент.
- **Value** — `Allow` / `Deny` (явный override) / отсутствие ноды.
  Три состояния, а не bool: "запрещено" и "ничего не сказано" — разное.
- **Groups** с наследованием: `owner → admin → manager → user`.
- **Weight** — приоритет группы при конфликте и одновременно ранг для
  проверок "кто выше".
- **Contexts** — условия активности ноды (`tenant=acme`, `env=prod`).
- **Temporary nodes** — `expires_at` на нодах и на членстве в группе.

Не реализовано намеренно: **tracks** (promote/demote по цепочке) и
**meta-ноды** (prefix/suffix, rate limits) — под текущие задачи API не нужны.

## 2. Доменная модель

Все идентификаторы и значения — newtype с валидацией при разборе, поэтому
невалидное состояние не доезжает до SQL и до резолвера.

| Тип | Инвариант |
|---|---|
| `PermissionKey` | 1..=256, сегменты через `.`, пустых сегментов нет, `*` — только целый сегмент, нижний регистр |
| `ContextKey` / `ContextValue` | ключ 1..=64 (`[a-z0-9_-]`, лоуэркейс), значение 1..=128 без control-символов |
| `GroupName` | 2..=64, лоуэркейс — в БД это FK по значению, а не по id |
| `GroupWeight` | `>= 0` |
| `ResourceType` / `ResourceId` | тип ресурса и его id (есть `From<Uuid>`) |
| `NodeExpiry` | момент истечения; прошлое допустимо — просроченная нода лежит в БД до чистки |
| `SourcePriority` | приоритет источника; `USER = 1_000_000` перебивает любой вес группы |

`api.ord*` и `api..read` отклоняются с ошибкой при разборе. Раньше такие
ключи молча не совпадали бы ни с чем — самый неприятный класс баг-репортов
"право выдал, а не работает".

Композитные типы: `PermissionNode` (поля приватные, `is_active_at(ctx, now)`
инъектирует время для тестов), `ContextSet` (`BTreeMap`, а не `Vec` пар —
ключи уникальны по определению, порядок стабилен, поэтому `cache_key()` не
клонирует и не сортирует), `RoleSnapshot` (список групп + максимальный вес,
чтобы не таскать `&[String]` и `i32` рядом позиционно), `ResourceRef`,
`AccessGrant`.

## 3. Три ортогональные проверки

Ключевая мысль всей системы. Для действия над чужим объектом или над другим
пользователем нужны **все три**, они отвечают на разные вопросы:

| Проверка | Вопрос | Где |
|---|---|---|
| RBAC | может ли роль вообще делать такое | `PermissionService::require` |
| ACL | открыт ли доступ к этой конкретной записи | `ResourceAcl` |
| Ранг | выше ли actor по весу, чем цель | `require_outranks` |

`profile.delete` без ранга дал бы manager-у удалить admin-а. `configuration.get`
без ACL открыл бы все чужие конфигурации.

## 4. Схема БД

Миграции `0003`–`0008`.

```sql
groups            (name PK-ish, weight, ...)
group_inheritance (group_name, parent_name, context)
permission_nodes  (holder_type IN ('user','group'), holder_id, node_key,
                   value, context JSONB, expires_at)
user_groups       (user_id, group_name, context, expires_at)
resource_access   (resource_type, resource_id,
                   holder_type IN ('user','role','min_weight'), holder_id,
                   mode, is_deny, is_public)
```

`holder_id` в `permission_nodes` — текст (`user_id::text` или `group.name`),
FK туда нет. Поэтому удаление группы делается транзакцией через
`PermissionAdmin::delete_group`, которая чистит `permission_nodes`,
`user_groups` и `group_inheritance` — иначе при повторном создании группы с тем
же именем её старые права "воскресли" бы.

`0004` добавляет уникальный индекс на ноды, и записи идут через
`ON CONFLICT ... DO UPDATE` — без этого повторная выдача права создавала бы
вторую противоречивую строку.

`0005` сидит группы `owner`(100) → `admin` → `manager` → `user` и права,
которые реально генерирует код (`configuration.get`, `instance.*`, `*` у
owner), а не абстрактные `api.orders.*`.

## 5. Резолвинг

Ноды выгружаются одним рекурсивным CTE (`user_direct_groups` → `group_tree`),
дальше вся логика — чистая функция `resolver::resolve_at`:

1. Отбросить неактивные: просроченные и с неподходящим контекстом.
2. Найти совпадения и посчитать специфичность: точный сегмент `+10`,
   wildcard `+1`. Поэтому `configuration.*` (score 11) строго перебивает `*`
   (score 1) — до правки они давали одинаковый score и результат зависел от
   порядка строк из Postgres.
3. При равной специфичности — выше `source_priority` (нода пользователя >
   группа с большим весом).
4. При полном равенстве побеждает **Deny**: детерминированно и fail-closed.
5. Совпадений нет — `Tristate::Undefined`.

`Undefined → запрет` — политика вызывающего кода (`as_bool(false)`), не часть
резолвера.

## 6. Кэш и инвалидация

`moka::future::Cache<CacheKey, Arc<Vec<PermissionNode>>>`, ключ —
`(user_id, hash(context))`, TTL 60 секунд (`app.rs: PERMISSION_CACHE_TTL`).

Ошибка загрузки **не кэшируется**. Иначе секундная недоступность БД
запретила бы пользователю всё на весь TTL.

- `invalidate_user(user_id)` — точечно, через `invalidate_entries_if`
  (раньше сбрасывался весь кэш, а аргумент отбрасывался).
- `invalidate_all()` — после правки прав ГРУППЫ: заранее неизвестно, кто
  входит в неё через наследование.

## 7. Поведение при сбое хранилища

- `check(...) -> bool` — fail-closed, при ошибке `false`.
- `require(...) -> PermissionResult<()>` — прокидывает `StoreError`, который
  маппится в **500**, а не в 403. Сбой БД не должен выглядеть как отказ в
  правах: это отправляет отладку по ложному следу.
- `InsufficientRank` — отдельный вариант ошибки, чтобы не путать "нет права"
  с "не хватает ранга".

## 8. Точка входа: `PermissionsFacade`

Хендлеры не работают с `PermissionService` напрямую — им нужна комбинация
проверок, и собирать её на каждом вызове легко забыть:

```rust
// RBAC на тип ресурса
perms.require_action(actor, &types.configuration, ResourceAction::Delete, &ctx).await?;

// RBAC + ACL на конкретную запись (учитывает is_public и bypass)
perms.require_resource_access(actor, &resource, action, is_public, &ctx).await?;

// RBAC + ранг: и то и другое обязательно
perms.require_action_on_user(actor, target, ...).await?;

// после создания/удаления объекта
perms.register_created_resource(author, &resource, share_with_peers).await?;
perms.cleanup_deleted_resource(&resource).await?;
```

`ResourceTypes` валидирует `configuration` / `instance` / `profile` один раз
при старте, чтобы в рантайме не парсить строки на каждый запрос.

В `is_allowed_with_bypass` персональный deny проверяется **до** bypass:
иначе право уровня роли молча перебивало бы точечный запрет, выставленный
администратором на конкретного пользователя.

## 9. Пайплайн запроса

```
Request → JWT → user_id + context (tenant, env)
        → PermissionsFacade
             ├─ RBAC:  cache.get_or_resolve(user_id, ctx) → resolve_at(key)
             ├─ ACL:   resource_access (deny → bypass → grants)
             └─ Ранг:  max_weight(actor) > max_weight(target)
        → Allow → handler
        → Deny  → 403 · StoreError → 500
```

## 10. Тесты

`cargo test -p rumary-perms` — 15 тестов: резолвинг (узкий wildcard против
корневого, deny при полном равенстве в обоих порядках, родительский ключ не
даёт права на дочерний, просроченная нода игнорируется), кэш (ошибка не
кэшируется как пустой набор, `require` не превращает сбой в отказ), ранги.

`cargo test -p rumary-dto --features domain_perms` — 17 тестов на инварианты
newtype и `ContextSet`.

Не покрыто тестами: SQL в `repo/perms.rs` и `ResourceAcl` — им нужна живая
Postgres. Проверены только компиляцией.

`cargo run -p rumary-perms --example usage` — сквозной сценарий на
`InMemoryStore`: кастомная роль, явный Deny поверх группового wildcard,
контексты, проверка ранга.
