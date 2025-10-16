use crate::model::BinaryContent;
use crate::repository::db::DBFileStoreBinaryRepository;
use c3p0::sqlx::{Postgres, Row, query};
use lightspeed_core::error::{ErrorCodes, LsError};
use sqlx::{AssertSqlSafe, PgConnection};
use std::borrow::Cow;

#[derive(Clone)]
pub struct PgFileStoreBinaryRepository {
    table_name: &'static str,
}

impl Default for PgFileStoreBinaryRepository {
    fn default() -> Self {
        PgFileStoreBinaryRepository { table_name: "LS_FILE_STORE_BINARY" }
    }
}

impl DBFileStoreBinaryRepository for PgFileStoreBinaryRepository {
    type DB = Postgres;

    async fn read_file(
        &self,
        tx: &mut PgConnection,
        repository_name: &str,
        file_path: &str,
    ) -> Result<BinaryContent<'_>, LsError> {
        let sql = format!("SELECT DATA FROM {} WHERE repository = $1 AND filepath = $2", self.table_name);

        let res = query(AssertSqlSafe(sql))
            .bind(repository_name)
            .bind(file_path)
            .fetch_one(tx.as_mut())
            .await
            .and_then(|row| row.try_get(0))
            .map(|content| BinaryContent::InMemory { content: Cow::Owned(content) })?;
        Ok(res)
    }

    async fn save_file<'a>(
        &self,
        tx: &mut PgConnection,
        repository_name: &str,
        file_path: &str,
        content: &'a BinaryContent<'a>,
    ) -> Result<u64, LsError> {
        let binary_content = match content {
            BinaryContent::InMemory { content } => Cow::Borrowed(content),
            BinaryContent::OpenDal { operator, path } => {
                let buffer = operator.read(path).await.map_err(|err| LsError::BadRequest {
                    message: format!("PgFileStoreBinaryRepository - Cannot read file [{path}]. Err: {err:?}"),
                    code: ErrorCodes::IO_ERROR,
                })?;
                Cow::Owned(Cow::Owned(buffer.to_vec()))
            }
        };

        let sql = format!("INSERT INTO {} (repository, filepath, data) VALUES ($1, $2, $3)", self.table_name);

        let res = query(AssertSqlSafe(sql))
            .bind(repository_name)
            .bind(file_path)
            .bind(binary_content.as_ref().as_ref())
            .execute(tx.as_mut())
            .await?;
        Ok(res.rows_affected())
    }

    async fn delete_file(&self, tx: &mut PgConnection, repository_name: &str, file_path: &str) -> Result<u64, LsError> {
        let sql = format!("DELETE FROM {} WHERE repository = $1 AND filepath = $2", self.table_name);
        let res =
            query(AssertSqlSafe(sql)).bind(repository_name).bind(file_path).execute(tx).await?;
        Ok(res.rows_affected())
    }
}
