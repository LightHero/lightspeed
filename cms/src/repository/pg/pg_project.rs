use crate::model::project::ProjectData;
use crate::repository::ProjectRepository;
use c3p0::sqlx::{*, error::into_c3p0_error};
use c3p0::*;
use lightspeed_core::error::LsError;
use ::sqlx::Row;
use std::ops::Deref;

#[derive(Clone)]
pub struct PgProjectRepository {
    repo: SqlxPgC3p0Json<ProjectData, DefaultJsonCodec>,
}

impl Deref for PgProjectRepository {
    type Target = SqlxPgC3p0Json<ProjectData, DefaultJsonCodec>;

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
    type Tx = PgTx;

    async fn fetch_by_id(&self, tx: &mut Self::Tx, id: i64) -> Result<Model<ProjectData>, LsError> {
        Ok(self.repo.fetch_one_by_id(tx, &id).await?)
    }

    async fn exists_by_name(&self, tx: &mut Self::Tx, name: &str) -> Result<bool, LsError> {
        let sql = r#"
        SELECT EXISTS (SELECT 1 from LS_CMS_PROJECT
            where LS_CMS_PROJECT.DATA ->> 'name' = $1)
        "#;
        let res = ::sqlx::query(sql).bind(name)
        .fetch_one(tx.conn())
        .await
        .and_then(|row| {row.try_get(0)        })
        .map_err(into_c3p0_error)?;
    Ok(res)
    }

    async fn save(
        &self,
        tx: &mut Self::Tx,
        model: NewModel<ProjectData>,
    ) -> Result<Model<ProjectData>, LsError> {
        Ok(self.repo.save(tx, model).await?)
    }

    async fn update(
        &self,
        tx: &mut Self::Tx,
        model: Model<ProjectData>,
    ) -> Result<Model<ProjectData>, LsError> {
        Ok(self.repo.update(tx, model).await?)
    }

    async fn delete(
        &self,
        tx: &mut Self::Tx,
        model: Model<ProjectData>,
    ) -> Result<Model<ProjectData>, LsError> {
        Ok(self.repo.delete(tx, model).await?)
    }
}
