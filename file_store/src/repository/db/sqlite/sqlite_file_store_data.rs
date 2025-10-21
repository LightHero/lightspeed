use crate::model::{FileStoreDataData, FileStoreDataModel};
use crate::repository::db::FileStoreDataRepository;
use c3p0::sql::OrderBy;
use c3p0::sqlx::{Row, Sqlite, query};
use c3p0::{sqlx::*, *};
use lightspeed_core::error::LsError;

#[derive(Clone, Default)]
pub struct SqliteFileStoreDataRepository {}

impl FileStoreDataRepository for SqliteFileStoreDataRepository {
    type DB = Sqlite;

    async fn exists_by_repository(
        &self,
        tx: &mut SqliteConnection,
        repository: &str,
        file_path: &str,
    ) -> Result<bool, LsError> {
        let sql = "SELECT EXISTS (SELECT 1 FROM LS_FILE_STORE_DATA WHERE (data ->> '$.repository') = ? AND (data ->> '$.file_path') = ?)";

        let res = query(sql).bind(repository).bind(file_path).fetch_one(tx).await.and_then(|row| row.try_get(0))?;
        Ok(res)
    }

    async fn fetch_one_by_id(&self, tx: &mut SqliteConnection, id: u64) -> Result<FileStoreDataModel, LsError> {
        Ok(tx.fetch_one_by_id(id).await?)
    }

    async fn fetch_one_by_repository(
        &self,
        tx: &mut SqliteConnection,
        repository: &str,
        file_path: &str,
    ) -> Result<FileStoreDataModel, LsError> {
        Ok(FileStoreDataModel::query_with(
            r#"
            WHERE (data ->> '$.repository') = ? AND (data ->> '$.file_path') = ?
        "#,
        )
        .bind(repository)
        .bind(file_path)
        .fetch_one(tx)
        .await?)
    }

    async fn fetch_all_by_repository(
        &self,
        tx: &mut SqliteConnection,
        repository: &str,
        offset: usize,
        max: usize,
        sort: OrderBy,
    ) -> Result<Vec<FileStoreDataModel>, LsError> {
        Ok(FileStoreDataModel::query_with(&format!(
            r#"
               WHERE (data ->> '$.repository') = ?
                order by id {}
                limit {}
                offset {}
               "#,
            sort, max, offset
        ))
        .bind(repository)
        .fetch_all(tx)
        .await?)
    }

    async fn save(
        &self,
        tx: &mut SqliteConnection,
        model: NewRecord<FileStoreDataData>,
    ) -> Result<FileStoreDataModel, LsError> {
        Ok(tx.save(model).await?)
    }

    async fn delete_by_id(&self, tx: &mut SqliteConnection, id: u64) -> Result<u64, LsError> {
        Ok(tx.delete_by_id::<FileStoreDataData>(id).await?)
    }
}
