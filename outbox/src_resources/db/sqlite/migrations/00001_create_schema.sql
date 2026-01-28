-- Your SQL goes here

-- ---------------------------
-- Begin - LS_OUTBOX_TASK -
-- ---------------------------

create table LS_OUTBOX_TASK (
    ID integer primary key autoincrement,
    VERSION integer not null,
    create_epoch_millis integer not null,
    update_epoch_millis integer not null,
    DATA JSON
);

-- End - LS_OUTBOX_TASK -
