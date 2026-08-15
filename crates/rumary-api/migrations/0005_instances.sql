-- InstanceRepository owns this table.
CREATE TABLE IF NOT EXISTS  instances (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    icon           TEXT,
    dir_name       TEXT NOT NULL UNIQUE,
    display_name   TEXT NOT NULL,
    version        TEXT NOT NULL,
    description    TEXT NOT NULL,
    loader         TEXT NOT NULL CHECK (loader IN ('vanilla', 'fabric', 'forge', 'neoforge')),
    loader_version TEXT,
    is_public      BOOLEAN NOT NULL DEFAULT false,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (char_length(dir_name) BETWEEN 1 AND 255),
    CHECK (char_length(display_name) BETWEEN 1 AND 255),
    CHECK (char_length(description) BETWEEN 1 AND 500),
    CHECK (version <> ''),
    CHECK (
        (loader = 'vanilla' AND loader_version IS NULL)
        OR (loader <> 'vanilla' AND loader_version IS NOT NULL AND loader_version <> '')
    )
);

CREATE INDEX idx_instances_public ON instances(id) WHERE is_public;

