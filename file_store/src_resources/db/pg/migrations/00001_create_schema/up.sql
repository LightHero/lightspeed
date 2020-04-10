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
    ID bigserial primary key,
    VERSION int not null,
    DATA JSONB
);

-- End - LS_FILE_STORE_BINARY -
