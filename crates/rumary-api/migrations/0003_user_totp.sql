-- TotpRepository owns this table.
CREATE TABLE user_totp
(
    user_id          UUID PRIMARY KEY REFERENCES users (user_id) ON DELETE CASCADE,
    encrypted_secret TEXT        NOT NULL,
    step             BIGINT        NOT NULL DEFAULT 0,
    nonce            TEXT        NOT NULL,
    confirmed        BOOLEAN     NOT NULL DEFAULT false,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (encrypted_secret <> ''),
    CHECK (nonce <> '')
);

