use crate::model::schema::SchemaData;
use crate::repository::SchemaRepository;
use c3p0::postgres::*;
use c3p0::*;
use lightspeed_core::error::LsError;
use std::ops::Deref;

#[derive(Clone)]
pub struct PgSchemaRepository {
    repo: PgC3p0Json<u64, i64, SchemaData, DefaultJsonCodec>,
}

impl Deref for PgSchemaRepository {
    type Target = PgC3p0Json<u64, i64, SchemaData, DefaultJsonCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl Default for PgSchemaRepository {
    fn default() -> Self {
        PgSchemaRepository { repo: PgC3p0JsonBuilder::new("LS_CMS_SCHEMA").build() }
    }
}

impl SchemaRepository for PgSchemaRepository {
    type Tx<'a> = PgTx<'a>;

    async fn fetch_by_id(&self, tx: &mut Self::Tx<'_>, id: u64) -> Result<Model<u64, SchemaData>, LsError> {
        Ok(self.repo.fetch_one_by_id(tx, &id).await?)
    }

    async fn exists_by_name_and_project_id(
        &self,
        tx: &mut Self::Tx<'_>,
        name: &str,
        project_id: u64,
    ) -> Result<bool, LsError> {
        let sql = r#"
        SELECT EXISTS (SELECT 1 from LS_CMS_SCHEMA
            where DATA ->> 'name' = $1 AND (DATA ->> 'project_id')::bigint = $2 )
        "#;

        let res = tx.fetch_one_value(sql, &[&name, &(project_id as i64)])
            .await?;

        // let res = ::sqlx::query(sql)
        //     .bind(name)
        //     .bind(project_id)
        //     .fetch_one(tx.conn())
        //     .await
        //     .and_then(|row| row.try_get(0))
        //     .map_err(into_c3p0_error)?;

        Ok(res)
    }

    async fn save(&self, tx: &mut Self::Tx<'_>, model: NewModel<SchemaData>) -> Result<Model<u64, SchemaData>, LsError> {
        Ok(self.repo.save(tx, model).await?)
    }

    async fn update(&self, tx: &mut Self::Tx<'_>, model: Model<u64, SchemaData>) -> Result<Model<u64, SchemaData>, LsError> {
        Ok(self.repo.update(tx, model).await?)
    }

    async fn delete(&self, tx: &mut Self::Tx<'_>, model: Model<u64, SchemaData>) -> Result<Model<u64, SchemaData>, LsError> {
        Ok(self.repo.delete(tx, model).await?)
    }

    async fn delete_by_project_id(&self, tx: &mut Self::Tx<'_>, project_id: u64) -> Result<u64, LsError> {
        let sql = r#"
            delete from LS_CMS_SCHEMA
            where (DATA ->> 'project_id')::bigint = $1
        "#;

        Ok(tx.execute(sql, &[&(project_id as i64)]).await?)

    }
}
