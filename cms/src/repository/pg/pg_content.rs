use crate::model::content::ContentData;
use crate::repository::ContentRepository;
use c3p0::postgres::*;
use c3p0::*;
use lightspeed_core::error::LsError;
use std::ops::Deref;

#[derive(Clone)]
pub struct PgContentRepository {
    repo: PgC3p0Json<u64, i64, ContentData, DefaultJsonCodec>,
}

impl PgContentRepository {
    pub fn new(table_name: &str) -> Self {
        Self { repo: PgC3p0JsonBuilder::new(table_name).build() }
    }
}

impl Deref for PgContentRepository {
    type Target = PgC3p0Json<u64, i64, ContentData, DefaultJsonCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl ContentRepository for PgContentRepository {
    type Tx<'a> = PgTx<'a>;

    async fn create_table(&self, tx: &mut Self::Tx<'_>) -> Result<(), LsError> {
        Ok(self.repo.create_table_if_not_exists(tx).await?)
    }

    async fn drop_table(&self, tx: &mut Self::Tx<'_>) -> Result<(), LsError> {
        Ok(self.repo.drop_table_if_exists(tx, true).await?)
    }

    async fn count_all(&self, tx: &mut Self::Tx<'_>) -> Result<u64, LsError> {
        Ok(self.repo.count_all(tx).await?)
    }

    async fn count_all_by_field_value(
        &self,
        tx: &mut Self::Tx<'_>,
        field_name: &str,
        field_value: &str,
    ) -> Result<u64, LsError> {
        let sql = format!(
            "SELECT COUNT(*) FROM {} WHERE  (DATA -> 'content' -> 'fields' -> '{}' -> 'value' ->> 'value') = $1 ",
            self.repo.queries().qualified_table_name,
            field_name
        );

        let res = tx.fetch_one_value(&sql, &[&field_value]).await
            .map(|val: i64| val as u64)
        ?;

        // let res = ::sqlx::query(&sql)
        //     .bind(field_value)
        //     .fetch_one(tx)
        //     .await
        //     .and_then(|row| row.try_get(0).map(|val: i64| val as u64))
        //     .map_err(into_c3p0_error)?;

        Ok(res)
    }

    async fn create_unique_constraint(
        &self,
        tx: &mut Self::Tx<'_>,
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

    async fn drop_unique_constraint(&self, tx: &mut Self::Tx<'_>, index_name: &str) -> Result<(), LsError> {
        Ok(tx.batch_execute(&format!("DROP INDEX {index_name} IF EXISTS")).await?)
    }

    async fn fetch_by_id(&self, tx: &mut Self::Tx<'_>, id: u64) -> Result<Model<u64, ContentData>, LsError> {
        Ok(self.repo.fetch_one_by_id(tx, &id).await?)
    }

    async fn save(&self, tx: &mut Self::Tx<'_>, model: NewModel<ContentData>) -> Result<Model<u64, ContentData>, LsError> {
        Ok(self.repo.save(tx, model).await?)
    }

    async fn update(&self, tx: &mut Self::Tx<'_>, model: Model<u64, ContentData>) -> Result<Model<u64, ContentData>, LsError> {
        Ok(self.repo.update(tx, model).await?)
    }

    async fn delete(&self, tx: &mut Self::Tx<'_>, model: Model<u64, ContentData>) -> Result<Model<u64, ContentData>, LsError> {
        Ok(self.repo.delete(tx, model).await?)
    }
}
