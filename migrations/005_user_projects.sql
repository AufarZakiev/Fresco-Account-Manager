CREATE TABLE user_projects (
    id                     BIGSERIAL PRIMARY KEY,
    user_id                BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    project_id             BIGINT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    project_authenticator  TEXT NOT NULL DEFAULT '',
    resource_share         REAL NOT NULL DEFAULT 100.0,
    suspended              BOOLEAN NOT NULL DEFAULT FALSE,
    dont_request_more_work BOOLEAN NOT NULL DEFAULT FALSE,
    no_rsc                 TEXT[] NOT NULL DEFAULT '{}',
    enrolled_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, project_id)
);

CREATE INDEX idx_user_projects_user_id ON user_projects(user_id);
