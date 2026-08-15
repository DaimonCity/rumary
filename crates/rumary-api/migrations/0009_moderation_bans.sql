-- ModerationRepository owns this table.
CREATE TABLE IF NOT EXISTS  moderation_bans (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_type  TEXT NOT NULL CHECK (subject_type IN ('account', 'device', 'ip_cidr')),
    account_id    UUID REFERENCES users(user_id),
    subject_hash  BYTEA,
    ip_network    CIDR,
    scope         TEXT NOT NULL CHECK (scope IN ('account', 'api', 'launcher', 'game')),
    starts_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at    TIMESTAMPTZ,
    reason_code   TEXT NOT NULL CHECK (char_length(reason_code) BETWEEN 1 AND 64),
    staff_note    TEXT CHECK (staff_note IS NULL OR char_length(staff_note) <= 2000),
    created_by    UUID NOT NULL REFERENCES users(user_id),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_by    UUID REFERENCES users(user_id),
    revoked_at    TIMESTAMPTZ,
    revoke_reason TEXT CHECK (
        revoke_reason IS NULL OR char_length(revoke_reason) BETWEEN 1 AND 500
    ),
    CHECK (expires_at IS NULL OR expires_at > starts_at),
    CHECK (
        (subject_type = 'account' AND account_id IS NOT NULL AND subject_hash IS NULL AND ip_network IS NULL)
        OR (subject_type = 'device' AND account_id IS NULL AND subject_hash IS NOT NULL AND ip_network IS NULL)
        OR (subject_type = 'ip_cidr' AND account_id IS NULL AND subject_hash IS NULL AND ip_network IS NOT NULL)
    ),
    CHECK (
        (revoked_at IS NULL AND revoked_by IS NULL AND revoke_reason IS NULL)
        OR (revoked_at IS NOT NULL AND revoked_by IS NOT NULL AND revoke_reason IS NOT NULL)
    )
);

CREATE INDEX IF NOT EXISTS idx_moderation_bans_active_account
    ON moderation_bans(account_id, scope, starts_at, expires_at)
    WHERE subject_type = 'account' AND revoked_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_moderation_bans_account_history
    ON moderation_bans(account_id, created_at DESC)
    WHERE subject_type = 'account';

CREATE INDEX IF NOT EXISTS idx_moderation_bans_device
    ON moderation_bans(subject_hash, starts_at, expires_at)
    WHERE subject_type = 'device' AND revoked_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_moderation_bans_ip
    ON moderation_bans USING gist(ip_network inet_ops)
    WHERE subject_type = 'ip_cidr' AND revoked_at IS NULL;

INSERT INTO permission_nodes (holder_type, holder_id, node_key, value) VALUES
    ('group', 'admin', 'user.ban', true),
    ('group', 'admin', 'user.ban.permanent', true),
    ('group', 'admin', 'user.unban', true);
