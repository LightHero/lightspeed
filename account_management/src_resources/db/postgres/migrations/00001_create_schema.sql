-- Your SQL goes here

-- ---------------------------
-- Begin - LS_AM_ACCOUNT -
-- ---------------------------

create table LS_AM_ACCOUNT (
    ID bigserial primary key,
    version bigint NOT NULL,
    create_time TIMESTAMPTZ NOT NULL,
    update_time TIMESTAMPTZ NOT NULL,
    data JSONB NOT NULL
);

CREATE UNIQUE INDEX LS_AM_ACCOUNT_UNIQUE_USERNAME ON LS_AM_ACCOUNT( (DATA->>'username') );
CREATE UNIQUE INDEX LS_AM_ACCOUNT_UNIQUE_EMAIL ON LS_AM_ACCOUNT( (DATA->>'email') );

-- End - LS_AM_ACCOUNT -


-- -------------------------
-- Begin - LS_AM_TOKEN -
-- -------------------------

create table LS_AM_TOKEN (
    ID bigserial primary key,
    version bigint NOT NULL,
    create_time TIMESTAMPTZ NOT NULL,
    update_time TIMESTAMPTZ NOT NULL,
    data JSONB NOT NULL
);

CREATE UNIQUE INDEX LS_AM_TOKEN_UNIQUE_TOKEN ON LS_AM_TOKEN( (DATA->>'token') );

CREATE INDEX LS_AM_TOKEN_EXPIRE_AT ON LS_AM_TOKEN( ((DATA->>'expire_at_epoch_seconds')::bigint) );

-- End - LS_AM_TOKEN -