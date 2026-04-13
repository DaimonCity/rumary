create table if not exists users (
    id uuid primary key,
    username text not null unique,
    password_hash text not null,
    auth_source jsonb not null,
    banned boolean not null default false,
    created_at timestamptz not null
);

create table if not exists sessions (
    token text primary key,
    user_id uuid not null references users(id) on delete cascade,
    issued_at timestamptz not null
);

create table if not exists clients (
    id uuid primary key,
    slug text not null unique,
    display_name text not null,
    minecraft_version text not null,
    authlib_injector_url text,
    files jsonb not null,
    rules jsonb not null,
    launch_arguments jsonb not null,
    created_at timestamptz not null
);

create table if not exists profiles (
    id uuid primary key,
    client_id uuid not null references clients(id) on delete cascade,
    slug text not null unique,
    display_name text not null,
    mods jsonb not null,
    rules jsonb not null,
    created_at timestamptz not null
);

create table if not exists launcher_builds (
    id uuid primary key,
    version text not null,
    channel text not null,
    download_url text not null,
    checksum text,
    changelog text,
    published_at timestamptz not null
);

create index if not exists idx_launcher_builds_channel_version on launcher_builds(channel, version);

create table if not exists installation_requests (
    id uuid primary key,
    user_id uuid not null references users(id) on delete cascade,
    client_id uuid not null references clients(id) on delete cascade,
    profile_id uuid references profiles(id) on delete set null,
    platform text not null,
    launcher_version text,
    created_at timestamptz not null
);

