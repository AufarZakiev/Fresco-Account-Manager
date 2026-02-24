CREATE TABLE hosts (
    id             BIGSERIAL PRIMARY KEY,
    user_id        BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    host_cpid      TEXT NOT NULL,
    domain_name    TEXT NOT NULL DEFAULT '',
    client_version TEXT NOT NULL DEFAULT '',
    platform_name  TEXT NOT NULL DEFAULT '',
    last_rpc_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    venue          TEXT NOT NULL DEFAULT '',
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, host_cpid)
);

CREATE INDEX idx_hosts_user_id ON hosts(user_id);
CREATE INDEX idx_hosts_cpid ON hosts(host_cpid);
