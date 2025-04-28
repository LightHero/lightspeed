-- Your SQL goes here

ALTER TABLE LS_AUTH_ACCOUNT ADD COLUMN create_epoch_millis bigint not null DEFAULT 0;
ALTER TABLE LS_AUTH_ACCOUNT ADD COLUMN update_epoch_millis bigint not null DEFAULT 0;

ALTER TABLE LS_AUTH_TOKEN ADD COLUMN create_epoch_millis bigint not null DEFAULT 0;
ALTER TABLE LS_AUTH_TOKEN ADD COLUMN update_epoch_millis bigint not null DEFAULT 0;
