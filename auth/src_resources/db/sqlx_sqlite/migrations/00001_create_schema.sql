-- Your SQL goes here

-- ---------------------------
-- Begin - LS_AUTH_ACCOUNT -
-- ---------------------------

create table LS_AUTH_ACCOUNT (
    ID integer primary key autoincrement,
    VERSION integer not null,
    create_epoch_millis integer not null,
    update_epoch_millis integer not null,
    DATA JSON
);

CREATE UNIQUE INDEX LS_AUTH_ACCOUNT_UNIQUE_USERNAME ON LS_AUTH_ACCOUNT( (DATA->>'$.username') );
CREATE UNIQUE INDEX LS_AUTH_ACCOUNT_UNIQUE_EMAIL ON LS_AUTH_ACCOUNT( (DATA->>'$.email') );

-- End - LS_AUTH_ACCOUNT -


-- -------------------------
-- Begin - LS_AUTH_TOKEN -
-- -------------------------

create table LS_AUTH_TOKEN (
    ID integer primary key autoincrement,
    VERSION integer not null,
    create_epoch_millis integer not null,
    update_epoch_millis integer not null,
    DATA JSON
);

CREATE UNIQUE INDEX LS_AUTH_TOKEN_UNIQUE_TOKEN ON LS_AUTH_TOKEN( (DATA->>'$.token') );

-- End - LS_AUTH_TOKEN -