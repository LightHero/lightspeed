-- Your SQL goes here

-- ---------------------------
-- Begin - LS_AUTH_ACCOUNT -
-- ---------------------------

create table LS_AUTH_ACCOUNT (
    ID bigserial primary key,
    version bigint NOT NULL,
    create_time TIMESTAMPTZ NOT NULL,
    update_time TIMESTAMPTZ NOT NULL,
    data JSONB NOT NULL
);

CREATE UNIQUE INDEX LS_AUTH_ACCOUNT_UNIQUE_USERNAME ON LS_AUTH_ACCOUNT( (DATA->>'username') );
CREATE UNIQUE INDEX LS_AUTH_ACCOUNT_UNIQUE_EMAIL ON LS_AUTH_ACCOUNT( (DATA->>'email') );

-- End - LS_AUTH_ACCOUNT -


-- -------------------------
-- Begin - LS_AUTH_TOKEN -
-- -------------------------

create table LS_AUTH_TOKEN (
    ID bigserial primary key,
    version bigint NOT NULL,
    create_time TIMESTAMPTZ NOT NULL,
    update_time TIMESTAMPTZ NOT NULL,
    data JSONB NOT NULL
);

CREATE UNIQUE INDEX LS_AUTH_TOKEN_UNIQUE_TOKEN ON LS_AUTH_TOKEN( (DATA->>'token') );

CREATE INDEX LS_AUTH_TOKEN_EXPIRE_AT ON LS_AUTH_TOKEN( ((DATA->>'expire_at_epoch_seconds')::bigint) );

-- End - LS_AUTH_TOKEN -