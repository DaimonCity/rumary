-- ResourceAclStore owns this polymorphic ACL table. Resource identifiers are
-- textual because one table protects several aggregate types.
CREATE TABLE resource_access (
    resource_type TEXT NOT NULL,
    resource_id   TEXT NOT NULL,
    holder_type   TEXT NOT NULL CHECK (holder_type IN ('user', 'role', 'min_weight')),
    holder_id     TEXT NOT NULL,
    value         BOOLEAN NOT NULL DEFAULT true,
    can_write     BOOLEAN NOT NULL DEFAULT false,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (resource_type, resource_id, holder_type, holder_id),
    CHECK (resource_type <> ''),
    CHECK (resource_id <> ''),
    CHECK (holder_id <> ''),
    CHECK (value OR NOT can_write)
);

CREATE INDEX idx_resource_access_lookup
    ON resource_access(resource_type, holder_type, holder_id);

