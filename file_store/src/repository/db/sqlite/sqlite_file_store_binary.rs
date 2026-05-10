use crate::model::BinaryContent;
use crate::repository::db::DBFileStoreBinaryRepository;
use c3p0::C3p0Pool;
use c3p0::SqliteC3p0Pool;
use c3p0::sqlx::{Row, Sqlite, query};
use futures::StreamExt;
use lightspeed_core::error::{ErrorCodes, LsError};
use sqlx::{AssertSqlSafe, SqliteConnection};
use std::borrow::Cow;

#[derive(Clone)]
pub struct SqliteFileStoreBinaryRepository {
    table_name: &'static str,
    pool: SqliteC3p0Pool,
}

impl SqliteFileStoreBinaryRepository {
    pub fn new(pool: SqliteC3p0Pool) -> Self {
        SqliteFileStoreBinaryRepository { table_name: "LS_FILE_STORE_BINARY", pool }
    }
}

impl DBFileStoreBinaryRepository for SqliteFileStoreBinaryRepository {
    type DB = Sqlite;

    async fn read_file(
        &self,
        tx: &mut SqliteConnection,
        repository_name: &str,
        file_path: &str,
    ) -> Result<BinaryContent<'_>, LsError> {
        let sql = format!("SELECT DATA FROM {} WHERE repository = ? AND filepath = ?", self.table_name);

        let res = query(AssertSqlSafe(sql))
            .bind(repository_name)
            .bind(file_path)
            .fetch_one(tx)
            .await
            .and_then(|row| row.try_get(0))
            .map(|content| BinaryContent::InMemory { content: Cow::Owned(content) })?;
        Ok(res)
    }

    /// SQLite has no stream support, so falls back to a buffered read inside
    /// our own transaction, returning an `InMemory` variant.
    async fn read_file_streamed(
        &self,
        repository_name: &str,
        file_path: &str,
    ) -> Result<BinaryContent<'static>, LsError> {
        let table_name = self.table_name;
        let repository = repository_name.to_owned();
        let path = file_path.to_owned();
        self.pool
            .transaction(async move |conn| {
                let sql = format!("SELECT DATA FROM {table_name} WHERE repository = ? AND filepath = ?");
                let row = query(AssertSqlSafe(sql)).bind(&repository).bind(&path).fetch_one(&mut *conn).await?;
                let bytes: Vec<u8> = row.try_get(0)?;
                Ok::<_, LsError>(BinaryContent::InMemory { content: Cow::Owned(bytes) })
            })
            .await
    }

    async fn save_file<'a>(
        &self,
        tx: &mut SqliteConnection,
        repository_name: &str,
        file_path: &str,
        content: &'a BinaryContent<'a>,
    ) -> Result<u64, LsError> {
        let binary_content: Cow<'_, [u8]> = match content {
            BinaryContent::InMemory { content } => Cow::Borrowed(content.as_ref()),
            BinaryContent::OpenDal { operator, path } => {
                let buffer = operator.read(path).await.map_err(|err| LsError::BadRequest {
                    message: format!("SqliteFileStoreBinaryRepository - Cannot read file [{path}]. Err: {err:?}"),
                    code: ErrorCodes::IO_ERROR,
                })?;
                Cow::Owned(buffer.to_vec())
            }
            BinaryContent::Stream { stream } => {
                let mut guard = stream.lock().await;
                let mut buf: Vec<u8> = Vec::new();
                while let Some(chunk) = guard.next().await {
                    buf.extend_from_slice(&chunk?);
                }
                Cow::Owned(buf)
            }
        };

        let sql = format!("INSERT INTO {} (repository, filepath, data) VALUES (?, ?, ?)", self.table_name);

        let res = query(AssertSqlSafe(sql))
            .bind(repository_name)
            .bind(file_path)
            .bind(binary_content.as_ref())
            .execute(tx)
            .await?;
        Ok(res.rows_affected())
    }

    async fn delete_file(
        &self,
        tx: &mut SqliteConnection,
        repository_name: &str,
        file_path: &str,
    ) -> Result<u64, LsError> {
        let sql = format!("DELETE FROM {} WHERE repository = ? AND filepath = ?", self.table_name);
        let res = query(AssertSqlSafe(sql)).bind(repository_name).bind(file_path).execute(tx).await?;
        Ok(res.rows_affected())
    }
}
