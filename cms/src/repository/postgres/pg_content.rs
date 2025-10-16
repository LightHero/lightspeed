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

    async fn count_all(&self, tx: &mut PgConnection) -> Result<u64, LsError> {
        Ok(tx.count_all::<ContentData>().await?)
    }

    async fn count_all_by_field_value(
        &self,
        tx: &mut PgConnection,
        field_name: &str,
        field_value: &str,
    ) -> Result<u64, LsError> {
        let sql = format!(
            "SELECT COUNT(*) FROM {} WHERE  (DATA -> 'content' -> 'fields' -> '{}' -> 'value' ->> 'value') = $1 ",
            ContentData::TABLE_NAME,
            field_name
        );

        Ok(sqlx::query(sqlx::AssertSqlSafe(sql))
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
