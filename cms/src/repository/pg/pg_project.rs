use crate::repository::ProjectRepository;
use c3p0::pg::*;
use c3p0::*;
use crate::model::project::ProjectData;
use std::ops::Deref;
use lightspeed_core::error::LightSpeedError;

#[derive(Clone)]
pub struct PgProjectRepository {
    repo: C3p0JsonPg<ProjectData, DefaultJsonCodec>,
}

impl Deref for PgProjectRepository {
    type Target = C3p0JsonPg<ProjectData, DefaultJsonCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl Default for PgProjectRepository {
    fn default() -> Self {
        PgProjectRepository {
            repo: C3p0JsonBuilder::new("CMS_PROJECT")
                .with_data_field_name("data_json")
                .build(),
        }
    }
}

impl ProjectRepository for PgProjectRepository {
    type CONN = PgConnection;

    fn fetch_by_id(&self, conn: &Self::CONN, id: i64) -> Result<Model<ProjectData>, LightSpeedError> {
        self.repo
            .fetch_one_by_id(conn, &id)?
            .ok_or_else(|| LightSpeedError::BadRequest {
                message: format!("No project found with id [{}]", id),
            })
    }

    fn save(&self, conn: &Self::CONN, model: NewModel<ProjectData>) -> Result<Model<ProjectData>, LightSpeedError> {
        Ok(self.repo.save(conn, model)?)
    }

    fn update(&self, conn: &Self::CONN, model: Model<ProjectData>) -> Result<Model<ProjectData>, LightSpeedError> {
        Ok(self.repo.update(conn, model)?)
    }

    fn delete(&self, conn: &Self::CONN, model: &Model<ProjectData>) -> Result<u64, LightSpeedError> {
        Ok(self.repo.delete(conn, model)?)
    }
}