-- Your SQL goes here

-- ---------------------------
-- Begin - LS_FILE_STORE_DATA -
-- ---------------------------

create table LS_FILE_STORE_DATA (
    id BIGINT PRIMARY KEY NOT NULL AUTO_INCREMENT,
    version INT NOT NULL,
    create_time TIMESTAMP(3) NOT NULL,
    update_time TIMESTAMP(3) NOT NULL,
    data JSON NOT NULL,
    UNIQUE INDEX LS_FILE_STORE_DATA_UNIQUE_REPOSITORY_FILEPATH ( (JSON_VALUE(DATA, '$.repository' RETURNING CHAR(255))), (JSON_VALUE(DATA, '$.file_path' RETURNING CHAR(255))))
);

-- End - LS_FILE_STORE_DATA -

-- ---------------------------------
-- Begin - LS_FILE_STORE_BINARY -
-- ---------------------------------

create table LS_FILE_STORE_BINARY (
    repository    TEXT NOT NULL,
    filepath      TEXT NOT NULL,
    data          LONGBLOB NOT NULL,
    UNIQUE INDEX LS_FILE_STORE_BINARY_UNIQUE_REPOSITORY_FILEPATH ( repository(255), filepath(255) )
);

-- End - LS_FILE_STORE_BINARY -
