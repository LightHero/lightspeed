use crate::model::project::ProjectData;
use crate::repository::ProjectRepository;
use c3p0::pg::*;
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use std::ops::Deref;

#[derive(Clone)]
pub struct PgProjectRepository {
    repo: PgC3p0Json<ProjectData, DefaultJsonCodec>,
}

impl Deref for PgProjectRepository {
    type Target = PgC3p0Json<ProjectData, DefaultJsonCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl Default for PgProjectRepository {
    fn default() -> Self {
        PgProjectRepository {
            repo: C3p0JsonBuilder::new("CMS_PROJECT").build(),
        }
    }
}

impl ProjectRepository for PgProjectRepository {
    type CONN = PgConnection;

    fn fetch_by_id(
        &self,
        conn: &Self::CONN,
        id: i64,
    ) -> Result<Model<ProjectData>, LightSpeedError> {
        self.repo
            .fetch_one_by_id(conn, &id)?
            .ok_or_else(|| LightSpeedError::BadRequest {
                message: format!("No project found with id [{}]", id),
            })
    }

    fn exists_by_name(&self, conn: &Self::CONN, name: &str) -> Result<bool, LightSpeedError> {
        let sql = r#"
            select count(*) from CMS_PROJECT
            where CMS_PROJECT.DATA ->> 'name' = $1
        "#;
        Ok(conn.fetch_one(sql, &[&name], |row| {
            let count: i64 = row.get(0);
            Ok(count > 0)
        })?)
    }

    fn save(
        &self,
        conn: &Self::CONN,
        model: NewModel<ProjectData>,
    ) -> Result<Model<ProjectData>, LightSpeedError> {
        Ok(self.repo.save(conn, model)?)
    }

    fn update(
        &self,
        conn: &Self::CONN,
        model: Model<ProjectData>,
    ) -> Result<Model<ProjectData>, LightSpeedError> {
        Ok(self.repo.update(conn, model)?)
    }

    fn delete(
        &self,
        conn: &Self::CONN,
        model: &Model<ProjectData>,
    ) -> Result<u64, LightSpeedError> {
        Ok(self.repo.delete(conn, model)?)
    }
}
