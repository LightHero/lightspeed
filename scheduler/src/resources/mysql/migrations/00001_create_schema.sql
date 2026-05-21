-- Your SQL goes here

-- -----------------------
-- Begin - LS_SCHEDULE -
-- -----------------------

create table LS_SCHEDULE (
    id BIGINT PRIMARY KEY NOT NULL AUTO_INCREMENT,
    version INT NOT NULL,
    create_time TIMESTAMP(3) NOT NULL,
    update_time TIMESTAMP(3) NOT NULL,
    data JSON NOT NULL
);

CREATE UNIQUE INDEX LS_SCHEDULE_UNIQUE_GROUP_NAME ON LS_SCHEDULE (
    (JSON_VALUE(data, '$.group_name' RETURNING CHAR(255))),
    (JSON_VALUE(data, '$.name'       RETURNING CHAR(255)))
);

CREATE INDEX LS_SCHEDULE_NEXT_RUN ON LS_SCHEDULE (
    (JSON_VALUE(data, '$.next_run_at_millis' RETURNING SIGNED))
);

-- End - LS_SCHEDULE -
