-- Your SQL goes here

-----------------------------
-- Begin - LS_FILE_STORE_DATA -
-----------------------------

create table LS_FILE_STORE_DATA (
    ID bigserial primary key,
    version bigint NOT NULL,
    create_time TIMESTAMPTZ NOT NULL,
    update_time TIMESTAMPTZ NOT NULL,
    data JSONB NOT NULL
);

CREATE UNIQUE INDEX LS_FILE_STORE_DATA_UNIQUE_REPOSITORY_FILEPATH ON LS_FILE_STORE_DATA( (data ->> 'repository'), (data ->> 'file_path') );

-- End - LS_FILE_STORE_DATA -

-----------------------------------
-- Begin - LS_FILE_STORE_BINARY -
-----------------------------------

-- `data` references a Postgres Large Object (pg_largeobject_metadata.oid).
-- Using LOs lets us stream content in / out via lo_open + lowrite/loread
-- instead of materializing the full payload as a single bytea bind, which
-- both avoids OOM on large uploads and bypasses the 1 GiB bytea limit.
-- Cleanup of the underlying LO on row delete is handled in
-- PgFileStoreBinaryRepository::delete_file via lo_unlink.
create table LS_FILE_STORE_BINARY (
    repository    TEXT NOT NULL,
    filepath      TEXT NOT NULL,
    data          OID  NOT NULL
);

CREATE UNIQUE INDEX LS_FILE_STORE_BINARY_UNIQUE_REPOSITORY_FILEPATH ON LS_FILE_STORE_BINARY( repository, filepath );

-- End - LS_FILE_STORE_BINARY -
