-- Your SQL goes here

-- ---------------------------
-- Begin - LS_OUTBOX_MESSAGE -
-- ---------------------------

create table LS_OUTBOX_MESSAGE (
    ID bigserial primary key,
    VERSION int not null,
    create_epoch_millis bigint not null,
    update_epoch_millis bigint not null,
    DATA JSONB
);

CREATE INDEX LS_OUTBOX_MESSAGE_INDEX_STATUS ON LS_OUTBOX_MESSAGE( (DATA->>'status') );
CREATE INDEX LS_OUTBOX_MESSAGE_INDEX_TYPE ON LS_OUTBOX_MESSAGE( (DATA->>'type') );

-- End - LS_OUTBOX_MESSAGE -

