-- Your SQL goes here

---------------------------
-- Begin - CMS_PROJECT -
---------------------------

create table CMS_PROJECT (
    ID bigserial primary key,
    VERSION int not null,
    DATA JSONB
);

CREATE UNIQUE INDEX CMS_PROJECT_UNIQUE_NAME ON CMS_PROJECT( (DATA->>'name') );

-- End - CMS_PROJECT -

---------------------------
-- Begin - CMS_SCHEMA -
---------------------------

create table CMS_SCHEMA (
    ID bigserial primary key,
    VERSION int not null,
    DATA JSONB
);

CREATE UNIQUE INDEX CMS_SCHEMA_UNIQUE_NAME_PROJECT_ID ON CMS_SCHEMA( (DATA->>'name'), (DATA->>'project_id') );

-- End - CMS_SCHEMA -
