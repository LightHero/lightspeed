-- Your SQL goes here

-- MySql does not support indexes composed by multiple columns from a JSON field :\

-- ALTER TABLE LS_FILE_STORE_DATA ADD INDEX LS_FILE_STORE_DATA_UNIQUE_REPOSITORY_FILEPATH(
--         (CAST(DATA->>"$.repository._json_tag" as CHAR(255)) COLLATE utf8mb4_bin),
--         (CAST(DATA->>"$.repository.repository_name" as CHAR(255)) COLLATE utf8mb4_bin),
--         (CAST(DATA->>"$.repository.file_path" as CHAR(255)) COLLATE utf8mb4_bin),
--         );
