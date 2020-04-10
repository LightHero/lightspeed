-- Your SQL goes here

-----------------------------
-- Begin - LS_FILE_STORAGE -
-----------------------------

create table LS_FILE_STORAGE (
    ID bigserial primary key,
    VERSION int not null,
    DATA JSONB
);

CREATE UNIQUE INDEX LS_FILE_STORAGE_UNIQUE_FILENAME ON LS_FILE_STORAGE( (DATA->>'filename') );

-- End - LS_FILE_STORAGE -
