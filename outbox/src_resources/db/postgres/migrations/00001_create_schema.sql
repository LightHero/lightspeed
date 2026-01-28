-- Your SQL goes here

-- ---------------------------
-- Begin - LS_OUTBOX_TASK -
-- ---------------------------

create table LS_OUTBOX_TASK (
    ID bigserial primary key,
    VERSION int not null,
    create_epoch_millis bigint not null,
    update_epoch_millis bigint not null,
    DATA JSONB
);

-- End - LS_OUTBOX_TASK -

