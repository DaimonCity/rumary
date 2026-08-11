-- DiscordUserRepository currently has no methods; this table establishes its
-- storage boundary for the future account-linking contract.
CREATE TABLE discord_users (
    user_id         UUID PRIMARY KEY REFERENCES users(user_id) ON DELETE CASCADE,
    discord_user_id TEXT NOT NULL UNIQUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (discord_user_id ~ '^[0-9]+$')
);

