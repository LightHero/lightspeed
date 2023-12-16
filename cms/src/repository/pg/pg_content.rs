use crate::model::content::ContentData;
use crate::repository::ContentRepository;
use c3p0::sqlx::{*, error::into_c3p0_error};
use c3p0::*;
use lightspeed_core::error::{ErrorCodes, LsError};
use ::sqlx::Row;
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
    type Conn = SqlxPgConnection;

    async fn create_table(&self, conn: &mut Self::Conn) -> Result<(), LsError> {
        Ok(self.repo.create_table_if_not_exists(conn).await?)
    }

    async fn drop_table(&self, conn: &mut Self::Conn) -> Result<(), LsError> {
        Ok(self.repo.drop_table_if_exists(conn, true).await?)
    }

    async fn count_all(&self, conn: &mut Self::Conn) -> Result<u64, LsError> {
        Ok(self.repo.count_all(conn).await?)
    }

    async fn count_all_by_field_value(
        &self,
        conn: &mut Self::Conn,
        field_name: &str,
        field_value: &str,
    ) -> Result<u64, LsError> {
        let sql = format!(
            "SELECT COUNT(*) FROM {} WHERE  (DATA -> 'content' -> 'fields' -> '{}' -> 'value' ->> 'value') = $1 ",
            self.repo.queries().qualified_table_name,
            field_name
        );
        let res = ::sqlx::query(&sql).bind(field_value)
        .fetch_one(&mut **conn.get_conn())
        .await
        .and_then(|row| {
            row.try_get(0).map(|val: i64| val as u64)
        })
        .map_err(into_c3p0_error)?;
    Ok(res)

        // Ok(conn
        //     .fetch_one_value(
        //         &format!(
        //     "SELECT COUNT(*) FROM {} WHERE  (DATA -> 'content' -> 'fields' -> '{}' -> 'value' ->> 'value') = $1 ",
        //     self.repo.queries().qualified_table_name,
        //     field_name
        // ),
        //         &[&field_value],
        //     )
        //     .await
        //     .map(|val: i64| val as u64)?)
    }

    async fn create_unique_constraint(
        &self,
        conn: &mut Self::Conn,
        index_name: &str,
        field_name: &str,
    ) -> Result<(), LsError> {
        Ok(conn
            .batch_execute(&format!(
                "CREATE UNIQUE INDEX {} ON {}( (DATA -> 'content' -> 'fields' -> '{}' -> 'value' ->> 'value') )",
                index_name,
                self.repo.queries().qualified_table_name,
                field_name
            ))
            .await?)
    }

    async fn drop_unique_constraint(&self, conn: &mut Self::Conn, index_name: &str) -> Result<(), LsError> {
        Ok(conn.batch_execute(&format!("DROP INDEX {index_name} IF EXISTS")).await?)
    }

    async fn fetch_by_id(&self, conn: &mut Self::Conn, id: i64) -> Result<Model<ContentData>, LsError> {
        Ok(self.repo.fetch_one_by_id(conn, &id).await?)
    }

    async fn save(
        &self,
        conn: &mut Self::Conn,
        model: NewModel<ContentData>,
    ) -> Result<Model<ContentData>, LsError> {
        Ok(self.repo.save(conn, model).await?)
    }

    async fn update(
        &self,
        conn: &mut Self::Conn,
        model: Model<ContentData>,
    ) -> Result<Model<ContentData>, LsError> {
        Ok(self.repo.update(conn, model).await?)
    }

    async fn delete(
        &self,
        conn: &mut Self::Conn,
        model: Model<ContentData>,
    ) -> Result<Model<ContentData>, LsError> {
        Ok(self.repo.delete(conn, model).await?)
    }
}
