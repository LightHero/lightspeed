use crate::model::schema::SchemaData;
use crate::repository::SchemaRepository;
use ::sqlx::Row;
use c3p0::sqlx::{error::into_c3p0_error, *};
use c3p0::*;
use lightspeed_core::error::LsError;
use std::ops::Deref;

#[derive(Clone)]
pub struct PgSchemaRepository {
    repo: SqlxPgC3p0Json<SchemaData, DefaultJsonCodec>,
}

impl Deref for PgSchemaRepository {
    type Target = SqlxPgC3p0Json<SchemaData, DefaultJsonCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl Default for PgSchemaRepository {
    fn default() -> Self {
        PgSchemaRepository { repo: C3p0JsonBuilder::new("LS_CMS_SCHEMA").build() }
    }
}

#[async_trait::async_trait]
impl SchemaRepository for PgSchemaRepository {
    type Tx = PgTx;

    async fn fetch_by_id(&self, tx: &mut Self::Tx, id: i64) -> Result<Model<SchemaData>, LsError> {
        Ok(self.repo.fetch_one_by_id(tx, &id).await?)
    }

    async fn exists_by_name_and_project_id(
        &self,
        tx: &mut Self::Tx,
        name: &str,
        project_id: i64,
    ) -> Result<bool, LsError> {
        let sql = r#"
        SELECT EXISTS (SELECT 1 from LS_CMS_SCHEMA
            where DATA ->> 'name' = $1 AND (DATA ->> 'project_id')::bigint = $2 )
        "#;

        let res = ::sqlx::query(sql)
            .bind(name)
            .bind(project_id)
            .fetch_one(tx.conn())
            .await
            .and_then(|row| row.try_get(0))
            .map_err(into_c3p0_error)?;
        Ok(res)
    }

    async fn save(&self, tx: &mut Self::Tx, model: NewModel<SchemaData>) -> Result<Model<SchemaData>, LsError> {
        Ok(self.repo.save(tx, model).await?)
    }

    async fn update(&self, tx: &mut Self::Tx, model: Model<SchemaData>) -> Result<Model<SchemaData>, LsError> {
        Ok(self.repo.update(tx, model).await?)
    }

    async fn delete(&self, tx: &mut Self::Tx, model: Model<SchemaData>) -> Result<Model<SchemaData>, LsError> {
        Ok(self.repo.delete(tx, model).await?)
    }

    async fn delete_by_project_id(&self, tx: &mut Self::Tx, project_id: i64) -> Result<u64, LsError> {
        let sql = r#"
            delete from LS_CMS_SCHEMA
            where (DATA ->> 'project_id')::bigint = $1
        "#;

        let res = ::sqlx::query(sql).bind(project_id).execute(tx.conn()).await.map_err(into_c3p0_error)?;
        Ok(res.rows_affected())
    }
}
