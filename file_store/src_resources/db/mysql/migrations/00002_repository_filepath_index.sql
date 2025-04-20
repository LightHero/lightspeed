-- Your SQL goes here

-- MySql does not support indexes composed by multiple columns from a JSON field 
-- So we cannot guarantee that the index is unique

-- ALTER TABLE LS_FILE_STORE_DATA ADD INDEX LS_FILE_STORE_DATA_UNIQUE_REPOSITORY_FILEPATH(
--         (CAST(DATA->>"$.repository._json_tag" as CHAR(255)) COLLATE utf8mb4_bin),
--         (CAST(DATA->>"$.repository.repository_name" as CHAR(255)) COLLATE utf8mb4_bin),
--         (CAST(DATA->>"$.repository.file_path" as CHAR(255)) COLLATE utf8mb4_bin),
--         );

-- CREATE UNIQUE INDEX LS_FILE_STORE_DATA_UNIQUE_REPOSITORY_FILEPATH ON LS_FILE_STORE_DATA (
--     CAST(JSON_ARRAY(
--         JSON_UNQUOTE(data->'$.repository._json_tag'),
--         JSON_UNQUOTE(data->'$.lastnrepository.repository_name')
--     )))
-- );