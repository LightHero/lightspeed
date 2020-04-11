-- Your SQL goes here

---------------------------------
-- Begin - LS_FILE_STORE_DATA -
---------------------------------

create table LS_FILE_STORE_DATA (
    ID bigserial primary key,
    VERSION int not null,
    DATA JSONB
);

CREATE UNIQUE INDEX LS_FILE_STORE_DATA_UNIQUE_FILENAME ON LS_FILE_STORE_DATA( (DATA->>'filename') );

-- End - LS_FILE_STORE_DATA -

-----------------------------------
-- Begin - LS_FILE_STORE_BINARY -
-----------------------------------

create table LS_FILE_STORE_BINARY (
    filename    TEXT NOT NULL,
    data  BYTEA
);

CREATE UNIQUE INDEX LS_FILE_STORE_BINARY_UNIQUE_FILENAME ON LS_FILE_STORE_BINARY( filename );

-- End - LS_FILE_STORE_BINARY -
