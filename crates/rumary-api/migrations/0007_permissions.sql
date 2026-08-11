-- The permissions subsystem is normalized separately: its traits operate on a
-- permission graph, so one aggregate requires several relational tables.
CREATE TABLE groups (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name       TEXT NOT NULL UNIQUE,
    weight     INTEGER NOT NULL DEFAULT 0 CHECK (weight >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (char_length(name) BETWEEN 2 AND 64),
    CHECK (name = lower(name)),
    CHECK (name ~ '^[a-z0-9_-]+$')
);

CREATE TABLE group_inheritance (
    group_id    UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    parent_name TEXT NOT NULL REFERENCES groups(name) ON DELETE CASCADE,
    context     JSONB NOT NULL DEFAULT '{}'::jsonb,
    PRIMARY KEY (group_id, parent_name, context),
    CHECK (jsonb_typeof(context) = 'object')
);

CREATE TABLE permission_nodes (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    holder_type TEXT NOT NULL CHECK (holder_type IN ('user', 'group')),
    holder_id   TEXT NOT NULL,
    node_key    TEXT NOT NULL,
    value       BOOLEAN NOT NULL,
    context     JSONB NOT NULL DEFAULT '{}'::jsonb,
    expires_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (holder_id <> ''),
    CHECK (node_key <> ''),
    CHECK (jsonb_typeof(context) = 'object'),
    UNIQUE (holder_type, holder_id, node_key, context)
);

CREATE INDEX idx_permission_nodes_holder
    ON permission_nodes(holder_type, holder_id);

CREATE TABLE user_groups (
    user_id    UUID NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    group_name TEXT NOT NULL REFERENCES groups(name) ON DELETE CASCADE,
    context    JSONB NOT NULL DEFAULT '{}'::jsonb,
    expires_at TIMESTAMPTZ,
    PRIMARY KEY (user_id, group_name, context),
    CHECK (jsonb_typeof(context) = 'object')
);

CREATE INDEX idx_user_groups_group_name ON user_groups(group_name);

INSERT INTO groups (name, weight) VALUES
    ('user', 0),
    ('moder', 10),
    ('admin', 20),
    ('owner', 100);

INSERT INTO group_inheritance (group_id, parent_name)
SELECT id, 'admin' FROM groups WHERE name = 'owner';

INSERT INTO group_inheritance (group_id, parent_name)
SELECT id, 'moder' FROM groups WHERE name = 'admin';

INSERT INTO group_inheritance (group_id, parent_name)
SELECT id, 'user' FROM groups WHERE name = 'moder';

INSERT INTO permission_nodes (holder_type, holder_id, node_key, value) VALUES
    ('group', 'user',  'auth.session.update', true),
    ('group', 'user',  'configuration.get', true),
    ('group', 'user',  'configuration.list', true),
    ('group', 'user',  'configuration.download', true),
    ('group', 'user',  'instance.get', true),
    ('group', 'user',  'instance.list', true),
    ('group', 'user',  'instance.configurations.list', true),
    ('group', 'user',  'user.get', true),
    ('group', 'user',  'group.get', true),
    ('group', 'user',  'group.list', true),
    ('group', 'admin', 'configuration.*', true),
    ('group', 'admin', 'instance.*', true),
    ('group', 'admin', 'group.*', true),
    ('group', 'admin', 'user.*', true),
    ('group', 'admin', 'settings.*', true),
    ('group', 'owner', '*', true);

