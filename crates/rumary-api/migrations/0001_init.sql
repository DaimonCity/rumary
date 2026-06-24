create table if not exists profiles
(
    id           uuid primary key,
    client_id    uuid        not null references clients (id) on delete cascade,
    slug         text        not null unique,
    display_name text        not null,
    mods         jsonb       not null,
    rules        jsonb       not null,
    created_at   timestamptz not null
);

create table if not exists clients
(
    id                uuid primary key,
    display_name      text        not null,
    icon              text        not null,
    minecraft_version text        not null,
    url               text        not null,
    loader            text        not null,
    loader_version    text        not null,
    launch_arguments  jsonb       not null -- ?
);