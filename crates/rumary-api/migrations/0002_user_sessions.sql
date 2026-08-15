-- SessionRepository owns this table. Its current contract allows one active
-- refresh session per user.
CREATE TABLE IF NOT EXISTS  user_sessions (
    user_id            UUID PRIMARY KEY REFERENCES users(user_id) ON DELETE CASCADE,
    token_id           UUID NOT NULL UNIQUE,
    refresh_token_hash TEXT NOT NULL,
    expires_at         TIMESTAMPTZ NOT NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (refresh_token_hash <> '')
);


CREATE INDEX IF NOT EXISTS  idx_user_sessions_expires_at ON user_sessions(expires_at);

