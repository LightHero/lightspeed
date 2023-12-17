use crate::model::content::ContentData;
use crate::repository::ContentRepository;
use ::sqlx::Row;
use c3p0::sqlx::{error::into_c3p0_error, *};
use c3p0::*;
use lightspeed_core::error::LsError;
use std::ops::Deref;

#[derive(Clone)]
pub struct PgContentRepository {
    repo: SqlxPgC3p0Json<ContentData, DefaultJsonCodec>,
}

impl PgContentRepository {
    pub fn new(table_name: &str) -> Self {
        Self { repo: C3p0JsonBuilder::new(table_name).build() }
    }
}

impl Deref for PgContentRepository {
    type Target = SqlxPgC3p0Json<ContentData, DefaultJsonCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

#[async_trait::async_trait]
impl ContentRepository for PgContentRepository {
    type Tx = PgTx;

    async fn create_table(&self, tx: &mut Self::Tx) -> Result<(), LsError> {
        Ok(self.repo.create_table_if_not_exists(tx).await?)
    }

    async fn drop_table(&self, tx: &mut Self::Tx) -> Result<(), LsError> {
        Ok(self.repo.drop_table_if_exists(tx, true).await?)
    }

    async fn count_all(&self, tx: &mut Self::Tx) -> Result<u64, LsError> {
        Ok(self.repo.count_all(tx).await?)
    }

    async fn count_all_by_field_value(
        &self,
        tx: &mut Self::Tx,
        field_name: &str,
        field_value: &str,
    ) -> Result<u64, LsError> {
        let sql = format!(
            "SELECT COUNT(*) FROM {} WHERE  (DATA -> 'content' -> 'fields' -> '{}' -> 'value' ->> 'value') = $1 ",
            self.repo.queries().qualified_table_name,
            field_name
        );
        let res = ::sqlx::query(&sql)
            .bind(field_value)
            .fetch_one(tx.conn())
            .await
            .and_then(|row| row.try_get(0).map(|val: i64| val as u64))
            .map_err(into_c3p0_error)?;
        Ok(res)
    }

    async fn create_unique_constraint(
        &self,
        tx: &mut Self::Tx,
        index_name: &str,
        field_name: &str,
    ) -> Result<(), LsError> {
        Ok(tx
            .batch_execute(&format!(
                "CREATE UNIQUE INDEX {} ON {}( (DATA -> 'content' -> 'fields' -> '{}' -> 'value' ->> 'value') )",
                index_name,
                self.repo.queries().qualified_table_name,
                field_name
            ))
            .await?)
    }

    async fn drop_unique_constraint(&self, tx: &mut Self::Tx, index_name: &str) -> Result<(), LsError> {
        Ok(tx.batch_execute(&format!("DROP INDEX {index_name} IF EXISTS")).await?)
    }

    async fn fetch_by_id(&self, tx: &mut Self::Tx, id: i64) -> Result<Model<ContentData>, LsError> {
        Ok(self.repo.fetch_one_by_id(tx, &id).await?)
    }

    async fn save(&self, tx: &mut Self::Tx, model: NewModel<ContentData>) -> Result<Model<ContentData>, LsError> {
        Ok(self.repo.save(tx, model).await?)
    }

    async fn update(&self, tx: &mut Self::Tx, model: Model<ContentData>) -> Result<Model<ContentData>, LsError> {
        Ok(self.repo.update(tx, model).await?)
    }

    async fn delete(&self, tx: &mut Self::Tx, model: Model<ContentData>) -> Result<Model<ContentData>, LsError> {
        Ok(self.repo.delete(tx, model).await?)
    }
}
