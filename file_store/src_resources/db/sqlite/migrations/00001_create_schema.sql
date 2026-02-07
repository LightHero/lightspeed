-- Your SQL goes here

-----------------------------
-- Begin - LS_FILE_STORE_DATA -
-----------------------------

create table LS_FILE_STORE_DATA (
    ID integer primary key autoincrement,
    VERSION integer not null,
    create_epoch_millis integer not null,
    update_epoch_millis integer not null,
    DATA JSON
);

CREATE UNIQUE INDEX LS_FILE_STORE_DATA_UNIQUE_REPOSITORY_FILEPATH ON LS_FILE_STORE_DATA( (DATA->>'$.repository'), (DATA->>'$.file_path') );

-- End - LS_FILE_STORE_DATA -


-- ---------------------------------
-- Begin - LS_FILE_STORE_BINARY -
-- ---------------------------------

create table LS_FILE_STORE_BINARY (
    repository    TEXT NOT NULL,
    filepath      TEXT NOT NULL,
    data          LONGBLOB NOT NULL
);

CREATE UNIQUE INDEX LS_FILE_STORE_BINARY_UNIQUE_REPOSITORY_FILEPATH ON LS_FILE_STORE_BINARY( repository, filepath );

-- End - LS_FILE_STORE_BINARY -