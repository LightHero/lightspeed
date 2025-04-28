-- Your SQL goes here

-- ---------------------------
-- Begin - LS_AUTH_ACCOUNT -
-- ---------------------------

create table LS_AUTH_ACCOUNT (
    ID BIGINT primary key NOT NULL AUTO_INCREMENT,
    VERSION int not null,
    create_epoch_millis bigint not null,
    update_epoch_millis bigint not null,
    DATA JSON
);

ALTER TABLE LS_AUTH_ACCOUNT
    ADD INDEX LS_AUTH_ACCOUNT_UNIQUE_USERNAME((
        CAST(DATA->>"$.username" as CHAR(255))
    COLLATE utf8mb4_bin
    ));

ALTER TABLE LS_AUTH_ACCOUNT
    ADD INDEX LS_AUTH_ACCOUNT_UNIQUE_EMAIL((
        CAST(DATA->>"$.email" as CHAR(255))
    COLLATE utf8mb4_bin
    ));

-- End - LS_AUTH_ACCOUNT -


-- -------------------------
-- Begin - LS_AUTH_TOKEN -
-- -------------------------

create table LS_AUTH_TOKEN (
    ID BIGINT primary key NOT NULL AUTO_INCREMENT,
    VERSION int not null,
    create_epoch_millis bigint not null,
    update_epoch_millis bigint not null,
    DATA JSON
);

ALTER TABLE LS_AUTH_TOKEN
    ADD INDEX LS_AUTH_TOKEN_UNIQUE_TOKEN((
        CAST(DATA->>"$.token" as CHAR(255))
    COLLATE utf8mb4_bin
    ));

-- End - LS_AUTH_TOKEN -