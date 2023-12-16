-- Your SQL goes here

CREATE UNIQUE INDEX LS_FILE_STORE_DATA_UNIQUE_REPOSITORY_FILEPATH ON LS_FILE_STORE_DATA( (data -> 'repository' ->> '_json_tag'), (data -> 'repository' ->> 'repository_name'), (data -> 'repository' ->> 'file_path') );
