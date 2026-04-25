create table if not exists players
(
    user_uuid      uuid unique ,
    minecraft_uuid uuid    not null,
    access_token   text    not null,
    nickname       text    not null,
    is_admin       boolean not null
);

create table if not exists users
(
    user_uuid          Uuid primary key default gen_random_uuid(),
    is_banned          bool,
    access_level       int,
    refresh_token_hash TEXT,
    expires_at         timestamptz,
    token_id           Uuid,
    token_version      int
);

create table if not exists minecraft_clients
(
    id           uuid primary key default gen_random_uuid(),
    name         TEXT,
    icon         path,
    url          text,
    version      TEXT,
    loader       TEXT,
    profiles     json,
    access_level int2
);

create table if not exists profiles
(
    id           uuid primary key default gen_random_uuid(),
    icon         path,
    name         text,
    hard_check   json,
    soft_check   json,
    access_level int2
)