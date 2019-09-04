-- Your SQL goes here

---------------------------
-- Begin - AUTH_ACCOUNT -
---------------------------

create table AUTH_ACCOUNT (
    ID bigserial primary key,
    VERSION int not null,
    DATA_JSON JSONB
);

CREATE UNIQUE INDEX AUTH_ACCOUNT_UNIQUE_USERNAME ON AUTH_ACCOUNT( (DATA_JSON->>'username') );
CREATE UNIQUE INDEX AUTH_ACCOUNT_UNIQUE_EMAIL ON AUTH_ACCOUNT( (DATA_JSON->>'email') );

-- End - AUTH_ACCOUNT -


---------------------------
-- Begin - AUTH_TOKEN -
---------------------------

create table AUTH_TOKEN (
    ID bigserial primary key,
    VERSION int not null,
    DATA_JSON JSONB
);

CREATE UNIQUE INDEX AUTH_TOKEN_TOKEN ON AUTH_TOKEN( (DATA_JSON->>'token') );

-- End - AUTH_TOKEN -