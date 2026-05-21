-- Your SQL goes here

-- -----------------------
-- Begin - LS_SCHEDULE -
-- -----------------------

create table LS_SCHEDULE (
    ID integer primary key autoincrement,
    version INTEGER NOT NULL,
    create_time TEXT NOT NULL,
    update_time TEXT NOT NULL,
    data JSON NOT NULL
);

CREATE UNIQUE INDEX LS_SCHEDULE_UNIQUE_GROUP_NAME ON LS_SCHEDULE (
    (DATA->>'$.group_name'),
    (DATA->>'$.name')
);

CREATE INDEX LS_SCHEDULE_NEXT_RUN ON LS_SCHEDULE (
    CAST(DATA->>'$.next_run_at_millis' AS INTEGER)
);

-- End - LS_SCHEDULE -
