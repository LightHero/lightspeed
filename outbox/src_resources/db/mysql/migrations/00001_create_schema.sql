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

ALTER TABLE LS_OUTBOX_MESSAGE
    ADD INDEX LS_OUTBOX_MESSAGE_INDEX_STATUS((
        CAST(DATA->>"$.status" as CHAR(255))
    COLLATE utf8mb4_bin
    ));
    
ALTER TABLE LS_OUTBOX_MESSAGE
    ADD INDEX LS_OUTBOX_MESSAGE_INDEX_TYPE((
        CAST(DATA->>"$.type" as CHAR(255))
    COLLATE utf8mb4_bin
    ));

-- End - LS_OUTBOX_MESSAGE -
