-- Your SQL goes here

-- ---------------------------
-- Begin - LS_OUTBOX_MESSAGE -
-- ---------------------------

create table LS_OUTBOX_MESSAGE (
    ID BIGINT primary key NOT NULL AUTO_INCREMENT,
    VERSION int not null,
    create_epoch_millis bigint not null,
    update_epoch_millis bigint not null,
    DATA JSON
);

-- End - LS_OUTBOX_MESSAGE -
