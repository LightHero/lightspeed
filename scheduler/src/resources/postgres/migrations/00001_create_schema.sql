-- Your SQL goes here

-- -----------------------
-- Begin - LS_SCHEDULE -
-- -----------------------

create table LS_SCHEDULE (
    ID bigserial primary key,
    version bigint NOT NULL,
    create_time TIMESTAMPTZ NOT NULL,
    update_time TIMESTAMPTZ NOT NULL,
    data JSONB NOT NULL
);

CREATE UNIQUE INDEX LS_SCHEDULE_UNIQUE_GROUP_NAME ON LS_SCHEDULE (
    (DATA->>'group_name'),
    (DATA->>'name')
);

CREATE INDEX LS_SCHEDULE_NEXT_RUN ON LS_SCHEDULE (
    ((DATA->>'next_run_at_millis')::bigint)
);

-- End - LS_SCHEDULE -
