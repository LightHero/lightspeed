-- Your SQL goes here

DROP INDEX IF EXISTS LS_FILE_STORE_DATA_UNIQUE_REPOSITORY_FILEPATH CASCADE;

CREATE UNIQUE INDEX LS_FILE_STORE_DATA_UNIQUE_REPOSITORY_FILEPATH ON LS_FILE_STORE_DATA( (data ->> 'repository'), (data ->> 'file_path') );
