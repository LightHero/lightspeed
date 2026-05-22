-- Your SQL goes here

-- ---------------------------
-- Begin - LS_AM_ACCOUNT -
-- ---------------------------

create table LS_AM_ACCOUNT (
    id BIGINT PRIMARY KEY NOT NULL AUTO_INCREMENT,
    version INT NOT NULL,
    create_time TIMESTAMP(3) NOT NULL,
    update_time TIMESTAMP(3) NOT NULL,
    data JSON NOT NULL
);

CREATE INDEX LS_AM_ACCOUNT_UNIQUE_USERNAME
    ON LS_AM_ACCOUNT ((JSON_VALUE(DATA, '$.username' RETURNING CHAR(255))));

CREATE INDEX LS_AM_ACCOUNT_UNIQUE_EMAIL
    ON LS_AM_ACCOUNT ((JSON_VALUE(DATA, '$.email' RETURNING CHAR(255))));

-- End - LS_AM_ACCOUNT -


-- -------------------------
-- Begin - LS_AM_TOKEN -
-- -------------------------

create table LS_AM_TOKEN (
    id BIGINT PRIMARY KEY NOT NULL AUTO_INCREMENT,
    version INT NOT NULL,
    create_time TIMESTAMP(3) NOT NULL,
    update_time TIMESTAMP(3) NOT NULL,
    data JSON NOT NULL
);

CREATE INDEX LS_AM_TOKEN_UNIQUE_TOKEN
    ON LS_AM_TOKEN ((JSON_VALUE(DATA, '$.token' RETURNING CHAR(255))));

CREATE INDEX LS_AM_TOKEN_EXPIRE_AT
    ON LS_AM_TOKEN ((JSON_VALUE(DATA, '$.expire_at_epoch_seconds' RETURNING SIGNED)));

-- End - LS_AM_TOKEN -