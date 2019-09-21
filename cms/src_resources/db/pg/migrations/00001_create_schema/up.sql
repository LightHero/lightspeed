-- Your SQL goes here

---------------------------
-- Begin - CMS_PROJECT -
---------------------------

create table CMS_PROJECT (
    ID bigserial primary key,
    VERSION int not null,
    DATA_JSON JSONB
);

CREATE UNIQUE INDEX CMS_PROJECT_UNIQUE_NAME ON CMS_PROJECT( (DATA_JSON->>'name') );

-- End - CMS_PROJECT -

---------------------------
-- Begin - CMS_SCHEMA -
---------------------------

create table CMS_SCHEMA (
    ID bigserial primary key,
    VERSION int not null,
    DATA_JSON JSONB
);

CREATE UNIQUE INDEX CMS_SCHEMA_UNIQUE_NAME_PROJECT_ID ON CMS_SCHEMA( (DATA_JSON->>'name', DATA_JSON->>'project_id') );

-- End - CMS_SCHEMA -

---------------------------
-- Begin - CMS_SCHEMA_CONTENT_MAPPING -
---------------------------

create table CMS_SCHEMA_CONTENT_MAPPING (
    ID bigserial primary key,
    VERSION int not null,
    DATA_JSON JSONB
);

CREATE UNIQUE INDEX CMS_SCHEMA_CONTENT_MAPPING_UNIQUE_SCHEMA_ID ON CMS_SCHEMA_CONTENT_MAPPING( (DATA_JSON->>'schema_id') );
CREATE UNIQUE INDEX CMS_SCHEMA_CONTENT_MAPPING_UNIQUE_CONTENT_TABLE ON CMS_SCHEMA_CONTENT_MAPPING( (DATA_JSON->>'content_table') );

-- End - CMS_SCHEMA_CONTENT_MAPPING -
