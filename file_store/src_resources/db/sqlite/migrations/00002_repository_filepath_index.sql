-- Your SQL goes here

CREATE UNIQUE INDEX LS_FILE_STORE_DATA_UNIQUE_REPOSITORY_FILEPATH ON LS_FILE_STORE_DATA( (DATA->>'$.repository._json_tag'), (DATA->>'$.repository.repository_name'), (DATA->>'$.repository.file_path') );