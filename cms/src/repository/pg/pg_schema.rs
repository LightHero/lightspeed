use crate::repository::SchemaRepository;
use c3p0::pg::*;
use c3p0::*;
use crate::model::schema::SchemaData;
use std::ops::Deref;

#[derive(Clone)]
pub struct PgSchemaRepository {
    repo: C3p0JsonPg<SchemaData, DefaultJsonCodec>,
}

impl Deref for PgSchemaRepository {
    type Target = C3p0JsonPg<SchemaData, DefaultJsonCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl Default for PgSchemaRepository {
    fn default() -> Self {
        PgSchemaRepository {
            repo: C3p0JsonBuilder::new("CMS_SCHEMA")
                .with_data_field_name("data_json")
                .build(),
        }
    }
}

impl SchemaRepository for PgSchemaRepository {
    type CONN = PgConnection;
}