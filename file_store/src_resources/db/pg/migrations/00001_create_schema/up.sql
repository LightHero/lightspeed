-- Your SQL goes here

-----------------------------
-- Begin - LS_FILE_STORE_DATA -
-----------------------------

create table LS_FILE_STORE_DATA (
    ID bigserial primary key,
    VERSION int not null,
    DATA JSONB
);

-- End - LS_FILE_STORE_DATA -

-----------------------------------
-- Begin - LS_FILE_STORE_BINARY -
-----------------------------------

create table LS_FILE_STORE_BINARY (
    repository    TEXT NOT NULL,
    filepath      TEXT NOT NULL,
    data          BYTEA
);

CREATE UNIQUE INDEX LS_FILE_STORE_BINARY_UNIQUE_REPOSITORY_FILEPATH ON LS_FILE_STORE_BINARY( repository, filepath );

-- End - LS_FILE_STORE_BINARY -
