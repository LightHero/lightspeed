use crate::repository::SchemaContentMappingRepository;
use c3p0::pg::*;
use c3p0::*;
use crate::model::schema_content_mapping::SchemaContentMappingData;

#[derive(Clone)]
pub struct PgSchemaContentMappingRepository {
    repo: C3p0JsonPg<SchemaContentMappingData, DefaultJsonCodec>,
}

impl Default for PgSchemaContentMappingRepository {
    fn default() -> Self {
        PgSchemaContentMappingRepository {
            repo: C3p0JsonBuilder::new("CMS_SCHEMA_CONTENT_MAPPING")
                .with_data_field_name("data_json")
                .build(),
        }
    }
}

impl SchemaContentMappingRepository for PgSchemaContentMappingRepository {
    type CONN = PgConnection;
}