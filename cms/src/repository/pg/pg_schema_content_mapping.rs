use crate::repository::SchemaContentMappingRepository;
use c3p0::pg::*;
use c3p0::*;
use crate::model::schema_content_mapping::SchemaContentMappingData;
use std::ops::Deref;
use lightspeed_core::error::LightSpeedError;

#[derive(Clone)]
pub struct PgSchemaContentMappingRepository {
    repo: C3p0JsonPg<SchemaContentMappingData, DefaultJsonCodec>,
}

impl Deref for PgSchemaContentMappingRepository {
    type Target = C3p0JsonPg<SchemaContentMappingData, DefaultJsonCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
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

    fn fetch_by_id(&self, conn: &Self::CONN, id: i64) -> Result<Model<SchemaContentMappingData>, LightSpeedError> {
        self.repo
            .fetch_one_by_id(conn, &id)?
            .ok_or_else(|| LightSpeedError::BadRequest {
                message: format!("No SchemaContentMapping found with id [{}]", id),
            })
    }

    fn save(&self, conn: &Self::CONN, model: NewModel<SchemaContentMappingData>) -> Result<Model<SchemaContentMappingData>, LightSpeedError> {
        Ok(self.repo.save(conn, model)?)
    }

    fn delete(&self, conn: &Self::CONN, model: &Model<SchemaContentMappingData>) -> Result<u64, LightSpeedError> {
        Ok(self.repo.delete(conn, model)?)
    }
}