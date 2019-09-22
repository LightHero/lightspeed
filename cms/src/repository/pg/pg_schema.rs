use crate::repository::SchemaRepository;
use c3p0::pg::*;
use c3p0::*;
use crate::model::schema::SchemaData;
use std::ops::Deref;
use lightspeed_core::error::LightSpeedError;

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

    fn fetch_by_id(&self, conn: &Self::CONN, id: i64) -> Result<Model<SchemaData>, LightSpeedError> {
        self.repo
            .fetch_one_by_id(conn, &id)?
            .ok_or_else(|| LightSpeedError::BadRequest {
                message: format!("No Schema found with id [{}]", id),
            })
    }

    fn save(&self, conn: &Self::CONN, model: NewModel<SchemaData>) -> Result<Model<SchemaData>, LightSpeedError> {
        Ok(self.repo.save(conn, model)?)
    }

    fn update(&self, conn: &Self::CONN, model: Model<SchemaData>) -> Result<Model<SchemaData>, LightSpeedError> {
        Ok(self.repo.update(conn, model)?)
    }

    fn delete(&self, conn: &Self::CONN, model: &Model<SchemaData>) -> Result<u64, LightSpeedError> {
        Ok(self.repo.delete(conn, model)?)
    }
}