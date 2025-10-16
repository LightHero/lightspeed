use crate::model::schema::SchemaData;
use crate::repository::SchemaRepository;
use c3p0::*;
use lightspeed_core::error::LsError;
use ::sqlx::{PgConnection, Postgres, Row};

#[derive(Clone)]
pub struct PostgresSchemaRepository {
}

impl Default for PostgresSchemaRepository {
    fn default() -> Self {
        PostgresSchemaRepository { }
    }
}

impl SchemaRepository for PostgresSchemaRepository {
    type DB = Postgres;

    async fn fetch_by_id(&self, tx: &mut PgConnection, id: u64) -> Result<Record<SchemaData>, LsError> {
        Ok(tx.fetch_one_by_id(id).await?)
    }

    async fn exists_by_name_and_project_id(
        &self,
        tx: &mut PgConnection,
        name: &str,
        project_id: u64,
    ) -> Result<bool, LsError> {
        let sql = r#"
        SELECT EXISTS (SELECT 1 from LS_CMS_SCHEMA
            where DATA ->> 'name' = $1 AND (DATA ->> 'project_id')::bigint = $2 )
        "#;

        let res = ::sqlx::query(sql)
            .bind(name)
            .bind(project_id as i64)
            .fetch_one(tx)
            .await
            .and_then(|row| row.try_get(0))?;

        Ok(res)
    }

    async fn save(
        &self,
        tx: &mut PgConnection,
        model: NewRecord<SchemaData>,
    ) -> Result<Record<SchemaData>, LsError> {
        Ok(tx.save(model).await?)
    }

    async fn update(
        &self,
        tx: &mut PgConnection,
        model: Record<SchemaData>,
    ) -> Result<Record<SchemaData>, LsError> {
        Ok(tx.update(model).await?)
    }

    async fn delete(
        &self,
        tx: &mut PgConnection,
        model: Record<SchemaData>,
    ) -> Result<Record<SchemaData>, LsError> {
        Ok(tx.delete(model).await?)
    }

    async fn delete_by_project_id(&self, tx: &mut PgConnection, project_id: u64) -> Result<u64, LsError> {
        let sql = r#"
            delete from LS_CMS_SCHEMA
            where (DATA ->> 'project_id')::bigint = $1
        "#;

        Ok(::sqlx::query(sql)
            .bind(project_id as i64)
            .execute(tx)
            .await?
            .rows_affected())

    }
}
