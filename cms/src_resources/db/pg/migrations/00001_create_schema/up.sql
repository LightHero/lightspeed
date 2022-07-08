-- Your SQL goes here

---------------------------
-- Begin - LS_CMS_PROJECT -
---------------------------

create table LS_CMS_PROJECT (
    ID bigserial primary key,
    VERSION int not null,
    create_epoch_millis bigint not null,
    update_epoch_millis bigint not null,
    DATA JSONB
);

CREATE UNIQUE INDEX LS_CMS_PROJECT_UNIQUE_NAME ON LS_CMS_PROJECT( (DATA->>'name') );

-- End - LS_CMS_PROJECT -

---------------------------
-- Begin - LS_CMS_SCHEMA -
---------------------------

create table LS_CMS_SCHEMA (
    ID bigserial primary key,
    VERSION int not null,
    create_epoch_millis bigint not null,
    update_epoch_millis bigint not null,
    DATA JSONB
);

CREATE UNIQUE INDEX LS_CMS_SCHEMA_UNIQUE_NAME_PROJECT_ID ON LS_CMS_SCHEMA( (DATA->>'name'), (DATA->>'project_id') );

-- End - LS_CMS_SCHEMA -
