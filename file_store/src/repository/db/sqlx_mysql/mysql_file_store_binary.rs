use crate::model::BinaryContent;
use crate::repository::db::DBFileStoreBinaryRepository;
use c3p0::sqlx::error::into_c3p0_error;
use c3p0::sqlx::sqlx::{MySql, Row, Transaction, query};
use lightspeed_core::error::{ErrorCodes, LsError};
use std::borrow::Cow;

#[derive(Clone)]
pub struct MySqlFileStoreBinaryRepository {
    table_name: &'static str,
}

impl Default for MySqlFileStoreBinaryRepository {
    fn default() -> Self {
        MySqlFileStoreBinaryRepository { table_name: "LS_FILE_STORE_BINARY" }
    }
}

impl DBFileStoreBinaryRepository for MySqlFileStoreBinaryRepository {
    type Tx<'a> = Transaction<'a, MySql>;

    async fn read_file(
        &self,
        tx: &mut Self::Tx<'_>,
        repository_name: &str,
        file_path: &str,
    ) -> Result<BinaryContent<'_>, LsError> {
        let sql = &format!("SELECT DATA FROM {} WHERE repository = ? AND filepath = ?", self.table_name);

        let res = query(sql)
            .bind(repository_name)
            .bind(file_path)
            .fetch_one(tx.as_mut())
            .await
            .and_then(|row| row.try_get(0))
            .map(|content| BinaryContent::InMemory { content: Cow::Owned(content) })
            .map_err(into_c3p0_error)?;
        Ok(res)
    }

    async fn save_file<'a>(
        &self,
        tx: &mut Self::Tx<'_>,
        repository_name: &str,
        file_path: &str,
        content: &'a BinaryContent<'a>,
    ) -> Result<u64, LsError> {
        let binary_content = match content {
            BinaryContent::InMemory { content } => Cow::Borrowed(content),
            BinaryContent::OpenDal { operator, path } => {
                let buffer = operator.read(path).await.map_err(|err| LsError::BadRequest {
                    message: format!("MySqlFileStoreBinaryRepository - Cannot read file [{path}]. Err: {err:?}"),
                    code: ErrorCodes::IO_ERROR,
                })?;
                Cow::Owned(Cow::Owned(buffer.to_vec()))
            }
        };

        let sql = &format!("INSERT INTO {} (repository, filepath, data) VALUES (?, ?, ?)", self.table_name);

        let res = query(sql)
            .bind(repository_name)
            .bind(file_path)
            .bind(binary_content.as_ref().as_ref())
            .execute(tx.as_mut())
            .await
            .map_err(into_c3p0_error)?;
        Ok(res.rows_affected())
    }

    async fn delete_file(&self, tx: &mut Self::Tx<'_>, repository_name: &str, file_path: &str) -> Result<u64, LsError> {
        let sql = &format!("DELETE FROM {} WHERE repository = ? AND filepath = ?", self.table_name);
        let res =
            query(sql).bind(repository_name).bind(file_path).execute(tx.as_mut()).await.map_err(into_c3p0_error)?;
        Ok(res.rows_affected())
    }
}
