use crate::model::content::ContentData;
use crate::repository::ContentRepository;
use c3p0::*;
use lightspeed_core::error::LsError;
use ::sqlx::{PgConnection, Postgres, Row};

#[derive(Clone)]
pub struct PostgresContentRepository {
}

impl PostgresContentRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl ContentRepository for PostgresContentRepository {
    type DB = Postgres;

    async fn count_all_by_schema(&self, tx: &mut PgConnection, schema_id: u64) -> Result<u64, LsError> {
        let sql = format!(
            "SELECT COUNT(*) FROM {} WHERE (DATA ->> 'schema_id')::bigint = $1",
            ContentData::TABLE_NAME
        );

        Ok(sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(schema_id as i64)
            .fetch_one(tx)
            .await
            .and_then(|row| row.try_get(0))
            .map(|val: i64| val as u64)?)
    }

    async fn count_all_by_schema_field_value(
        &self,
        tx: &mut PgConnection,
        schema_id: u64,
        field_name: &str,
        field_value: &str,
    ) -> Result<u64, LsError> {
        let sql = format!(
            "SELECT COUNT(*) FROM {} WHERE (DATA ->> 'schema_id')::bigint = $1 AND (DATA -> 'content' -> 'fields' -> '{}' -> 'value' ->> 'value') = $2 ",
            ContentData::TABLE_NAME,
            field_name
        );

        Ok(sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(schema_id as i64)
            .bind(field_value)
            .fetch_one(tx)
            .await
            .and_then(|row| row.try_get(0))
            .map(|val: i64| val as u64)?)

    }

    async fn fetch_by_id(&self, tx: &mut PgConnection, id: u64) -> Result<Record<ContentData>, LsError> {
        Ok(tx.fetch_one_by_id(id).await?)
    }

    async fn save(
        &self,
        tx: &mut PgConnection,
        model: NewRecord<ContentData>,
    ) -> Result<Record<ContentData>, LsError> {
        Ok(tx.save(model).await?)
    }

    async fn update(
        &self,
        tx: &mut PgConnection,
        model: Record<ContentData>,
    ) -> Result<Record<ContentData>, LsError> {
        Ok(tx.update(model).await?)
    }

    async fn delete(
        &self,
        tx: &mut PgConnection,
        model: Record<ContentData>,
    ) -> Result<Record<ContentData>, LsError> {
        Ok(tx.delete(model).await?)
    }
}
