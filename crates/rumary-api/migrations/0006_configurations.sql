-- ConfigurationRepository owns this table.
CREATE TABLE IF NOT EXISTS  configurations (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    icon         TEXT,
    dir_name     TEXT NOT NULL,
    display_name TEXT NOT NULL,
    instance_id  UUID NOT NULL REFERENCES instances(id) ON DELETE CASCADE,
    is_public    BOOLEAN NOT NULL DEFAULT false,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (char_length(dir_name) BETWEEN 1 AND 255),
    CHECK (char_length(display_name) BETWEEN 1 AND 255),
    UNIQUE (instance_id, dir_name)
);

CREATE INDEX idx_configurations_instance ON configurations(instance_id);
CREATE INDEX idx_configurations_public ON configurations(instance_id, id) WHERE is_public;

