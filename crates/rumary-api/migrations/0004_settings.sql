-- SettingsRepository owns this singleton table.
CREATE TABLE IF NOT EXISTS  settings (
    singleton         BOOLEAN PRIMARY KEY DEFAULT true CHECK (singleton),
    instances_dir_path TEXT NOT NULL,
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (instances_dir_path <> '')
);

