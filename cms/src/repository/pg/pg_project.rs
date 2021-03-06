use crate::model::project::ProjectData;
use crate::repository::ProjectRepository;
use c3p0::postgres::*;
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
        PgProjectRepository { repo: C3p0JsonBuilder::new("LS_CMS_PROJECT").build() }
    }
}

#[async_trait::async_trait]
impl ProjectRepository for PgProjectRepository {
    type Conn = PgConnection;

    async fn fetch_by_id(&self, conn: &mut Self::Conn, id: i64) -> Result<Model<ProjectData>, LightSpeedError> {
        Ok(self.repo.fetch_one_by_id(conn, &id).await?)
    }

    async fn exists_by_name(&self, conn: &mut Self::Conn, name: &str) -> Result<bool, LightSpeedError> {
        let sql = r#"
            select count(*) from LS_CMS_PROJECT
            where LS_CMS_PROJECT.DATA ->> 'name' = $1
        "#;
        Ok(conn
            .fetch_one(sql, &[&name], |row| {
                let count: i64 = row.get(0);
                Ok(count > 0)
            })
            .await?)
    }

    async fn save(
        &self,
        conn: &mut Self::Conn,
        model: NewModel<ProjectData>,
    ) -> Result<Model<ProjectData>, LightSpeedError> {
        Ok(self.repo.save(conn, model).await?)
    }

    async fn update(
        &self,
        conn: &mut Self::Conn,
        model: Model<ProjectData>,
    ) -> Result<Model<ProjectData>, LightSpeedError> {
        Ok(self.repo.update(conn, model).await?)
    }

    async fn delete(
        &self,
        conn: &mut Self::Conn,
        model: Model<ProjectData>,
    ) -> Result<Model<ProjectData>, LightSpeedError> {
        Ok(self.repo.delete(conn, model).await?)
    }
}
