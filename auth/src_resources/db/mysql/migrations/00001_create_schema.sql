-- Your SQL goes here

-- ---------------------------
-- Begin - LS_AUTH_ACCOUNT -
-- ---------------------------

create table LS_AUTH_ACCOUNT (
    ID BIGINT primary key NOT NULL AUTO_INCREMENT,
    VERSION int not null,
    create_epoch_millis bigint not null,
    update_epoch_millis bigint not null,
    DATA JSON,
    UNIQUE INDEX LS_AUTH_ACCOUNT_UNIQUE_USERNAME ( (JSON_VALUE(DATA, '$.username' RETURNING CHAR(255))) ),
    UNIQUE INDEX LS_AUTH_ACCOUNT_UNIQUE_EMAIL ( (JSON_VALUE(DATA, '$.email' RETURNING CHAR(255))) )
);

-- End - LS_AUTH_ACCOUNT -


-- -------------------------
-- Begin - LS_AUTH_TOKEN -
-- -------------------------

create table LS_AUTH_TOKEN (
    ID BIGINT primary key NOT NULL AUTO_INCREMENT,
    VERSION int not null,
    create_epoch_millis bigint not null,
    update_epoch_millis bigint not null,
    DATA JSON,
    UNIQUE INDEX LS_AUTH_TOKEN_UNIQUE_TOKEN ( (JSON_VALUE(DATA, '$.token' RETURNING CHAR(255))) )
);

-- End - LS_AUTH_TOKEN -