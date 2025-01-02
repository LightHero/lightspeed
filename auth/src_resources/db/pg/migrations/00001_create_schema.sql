-- Your SQL goes here

-- ---------------------------
-- Begin - LS_AUTH_ACCOUNT -
-- ---------------------------

create table LS_AUTH_ACCOUNT (
    ID bigserial primary key,
    VERSION int not null,
    create_epoch_millis bigint not null,
    update_epoch_millis bigint not null,
    DATA JSONB
);

CREATE UNIQUE INDEX LS_AUTH_ACCOUNT_UNIQUE_USERNAME ON LS_AUTH_ACCOUNT( (DATA->>'username') );
CREATE UNIQUE INDEX LS_AUTH_ACCOUNT_UNIQUE_EMAIL ON LS_AUTH_ACCOUNT( (DATA->>'email') );

-- End - LS_AUTH_ACCOUNT -


-- -------------------------
-- Begin - LS_AUTH_TOKEN -
-- -------------------------

create table LS_AUTH_TOKEN (
    ID bigserial primary key,
    VERSION int not null,
    create_epoch_millis bigint not null,
    update_epoch_millis bigint not null,
    DATA JSONB
);

CREATE UNIQUE INDEX LS_AUTH_TOKEN_UNIQUE_TOKEN ON LS_AUTH_TOKEN( (DATA->>'token') );

-- End - LS_AUTH_TOKEN -