-- Your SQL goes here

-- ---------------------------
-- Begin - LS_OUTBOX_MESSAGE -
-- ---------------------------

create table LS_OUTBOX_MESSAGE (
    ID BIGINT primary key NOT NULL AUTO_INCREMENT,
    VERSION int not null,
    create_epoch_millis bigint not null,
    update_epoch_millis bigint not null,
    DATA JSON,
    INDEX LS_OUTBOX_MESSAGE_INDEX_STATUS ( (JSON_VALUE(DATA, '$.status' RETURNING CHAR(255))) ),
    INDEX LS_OUTBOX_MESSAGE_INDEX_TYPE ( (JSON_VALUE(DATA, '$.type' RETURNING CHAR(255))) )
);

-- End - LS_OUTBOX_MESSAGE -
