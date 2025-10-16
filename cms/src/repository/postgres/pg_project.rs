use crate::model::project::ProjectData;
use crate::repository::ProjectRepository;
use ::sqlx::{PgConnection, Postgres, Row};
use c3p0::*;
use lightspeed_core::error::LsError;

#[derive(Clone, Default)]
pub struct PostgresProjectRepository {}

impl ProjectRepository for PostgresProjectRepository {
    type DB = Postgres;

    async fn fetch_by_id(&self, tx: &mut PgConnection, id: u64) -> Result<Record<ProjectData>, LsError> {
        Ok(tx.fetch_one_by_id(id).await?)
    }

    async fn exists_by_name(&self, tx: &mut PgConnection, name: &str) -> Result<bool, LsError> {
        let sql = r#"
        SELECT EXISTS (SELECT 1 from LS_CMS_PROJECT
            where LS_CMS_PROJECT.DATA ->> 'name' = $1)
        "#;

        let res = ::sqlx::query(sql).bind(name).fetch_one(tx).await.and_then(|row| row.try_get(0))?;

        Ok(res)
    }

    async fn save(&self, tx: &mut PgConnection, model: NewRecord<ProjectData>) -> Result<Record<ProjectData>, LsError> {
        Ok(tx.save(model).await?)
    }

    async fn update(&self, tx: &mut PgConnection, model: Record<ProjectData>) -> Result<Record<ProjectData>, LsError> {
        Ok(tx.update(model).await?)
    }

    async fn delete(&self, tx: &mut PgConnection, model: Record<ProjectData>) -> Result<Record<ProjectData>, LsError> {
        Ok(tx.delete(model).await?)
    }
}
