use crate::model::project::ProjectData;
use crate::repository::ProjectRepository;
use c3p0::postgres::*;
use c3p0::*;
use lightspeed_core::error::LsError;
use std::ops::Deref;

#[derive(Clone)]
pub struct PostgresProjectRepository {
    repo: PgC3p0Json<u64, i64, ProjectData, DefaultJsonCodec>,
}

impl Deref for PostgresProjectRepository {
    type Target = PgC3p0Json<u64, i64, ProjectData, DefaultJsonCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl Default for PostgresProjectRepository {
    fn default() -> Self {
        PostgresProjectRepository { repo: PgC3p0JsonBuilder::new("LS_CMS_PROJECT").build() }
    }
}

impl ProjectRepository for PostgresProjectRepository {
    type Tx<'a> = PgTx<'a>;

    async fn fetch_by_id(&self, tx: &mut Self::Tx<'_>, id: u64) -> Result<Model<u64, ProjectData>, LsError> {
        Ok(self.repo.fetch_one_by_id(tx, &id).await?)
    }

    async fn exists_by_name(&self, tx: &mut Self::Tx<'_>, name: &str) -> Result<bool, LsError> {
        let sql = r#"
        SELECT EXISTS (SELECT 1 from LS_CMS_PROJECT
            where LS_CMS_PROJECT.DATA ->> 'name' = $1)
        "#;

        let res = tx.fetch_one_value(sql, &[&name]).await?;

        // let res = ::sqlx::query(sql)
        //     .bind(name)
        //     .fetch_one(tx.conn())
        //     .await
        //     .and_then(|row| row.try_get(0))
        //     .map_err(into_c3p0_error)?;
        Ok(res)
    }

    async fn save(
        &self,
        tx: &mut Self::Tx<'_>,
        model: NewModel<ProjectData>,
    ) -> Result<Model<u64, ProjectData>, LsError> {
        Ok(self.repo.save(tx, model).await?)
    }

    async fn update(
        &self,
        tx: &mut Self::Tx<'_>,
        model: Model<u64, ProjectData>,
    ) -> Result<Model<u64, ProjectData>, LsError> {
        Ok(self.repo.update(tx, model).await?)
    }

    async fn delete(
        &self,
        tx: &mut Self::Tx<'_>,
        model: Model<u64, ProjectData>,
    ) -> Result<Model<u64, ProjectData>, LsError> {
        Ok(self.repo.delete(tx, model).await?)
    }
}
