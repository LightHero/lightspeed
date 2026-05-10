-- Your SQL goes here

-----------------------------
-- Begin - LS_FILE_STORE_DATA -
-----------------------------

create table LS_FILE_STORE_DATA (
    ID integer primary key autoincrement,
    version INTEGER NOT NULL,
    create_time TEXT NOT NULL,
    update_time TEXT NOT NULL,
    data JSON NOT NULL CHECK (json_valid(data))
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