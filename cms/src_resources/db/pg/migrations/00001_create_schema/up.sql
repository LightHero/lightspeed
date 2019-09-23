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

---------------------------
-- Begin - CMS_SCHEMA_CONTENT_MAPPING -
---------------------------

create table CMS_SCHEMA_CONTENT_MAPPING (
    ID bigserial primary key,
    VERSION int not null,
    DATA JSONB
);

CREATE UNIQUE INDEX CMS_SCHEMA_CONTENT_MAPPING_UNIQUE_SCHEMA_ID ON CMS_SCHEMA_CONTENT_MAPPING( (DATA->>'schema_id') );
CREATE UNIQUE INDEX CMS_SCHEMA_CONTENT_MAPPING_UNIQUE_CONTENT_TABLE ON CMS_SCHEMA_CONTENT_MAPPING( (DATA->>'content_table') );

-- End - CMS_SCHEMA_CONTENT_MAPPING -
