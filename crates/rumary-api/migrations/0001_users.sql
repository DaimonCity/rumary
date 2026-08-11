-- UserRepository owns this table. Authentication sessions, TOTP credentials,
-- moderation state and external identities live in their repository tables.
CREATE TABLE users (
    user_id     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    login         TEXT NOT NULL UNIQUE,
    nickname      TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    access_level  INTEGER NOT NULL DEFAULT 0 CHECK (access_level BETWEEN 0 AND 65535),
    token_version INTEGER NOT NULL DEFAULT 0 CHECK (token_version >= 0),
    is_public     BOOLEAN NOT NULL DEFAULT true,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (char_length(login) BETWEEN 3 AND 20),
    CHECK (char_length(nickname) BETWEEN 3 AND 16),
    CHECK (password_hash <> '')
);

