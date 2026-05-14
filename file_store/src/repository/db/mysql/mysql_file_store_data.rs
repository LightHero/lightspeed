use crate::error::LsFileStoreError;
use crate::model::{FileStoreDataData, FileStoreDataModel};
use crate::repository::db::FileStoreDataRepository;
use c3p0::sql::OrderBy;
use c3p0::{sqlx::*, *};

#[derive(Clone, Default)]
pub struct MySqlFileStoreDataRepository {}

impl FileStoreDataRepository for MySqlFileStoreDataRepository {
    type DB = MySql;

    async fn exists_by_repository(
        &self,
        tx: &mut MySqlConnection,
        repository: &str,
        file_path: &str,
    ) -> Result<bool, LsFileStoreError> {
        let sql = "SELECT EXISTS (SELECT 1 FROM LS_FILE_STORE_DATA WHERE JSON_VALUE(data, '$.repository' RETURNING CHAR(255)) = ? AND JSON_VALUE(data, '$.file_path' RETURNING CHAR(255)) = ?)";

        let res = query(sql).bind(repository).bind(file_path).fetch_one(tx).await.and_then(|row| row.try_get(0))?;
        Ok(res)
    }

    async fn fetch_one_by_id(&self, tx: &mut MySqlConnection, id: i64) -> Result<FileStoreDataModel, LsFileStoreError> {
        Ok(tx.fetch_one_by_id::<FileStoreDataData>(id).await?)
    }

    async fn fetch_one_by_repository(
        &self,
        tx: &mut MySqlConnection,
        repository: &str,
        file_path: &str,
    ) -> Result<FileStoreDataModel, LsFileStoreError> {
        Ok(FileStoreDataModel::query_with_tail(
            r#"
            WHERE JSON_VALUE(data, '$.repository' RETURNING CHAR(255)) = ? AND JSON_VALUE(data, '$.file_path' RETURNING CHAR(255)) = ?
        "#,
        )
        .bind(repository)
        .bind(file_path)
        .fetch_one(tx)
        .await?)
    }

    async fn fetch_all_by_repository(
        &self,
        tx: &mut MySqlConnection,
        repository: &str,
        offset: usize,
        max: usize,
        sort: OrderBy,
    ) -> Result<Vec<FileStoreDataModel>, LsFileStoreError> {
        Ok(FileStoreDataModel::query_with_tail(&format!(
            r#"
               WHERE JSON_VALUE(data, '$.repository' RETURNING CHAR(255)) = ?
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
        tx: &mut MySqlConnection,
        model: NewRecord<FileStoreDataData>,
    ) -> Result<FileStoreDataModel, LsFileStoreError> {
        Ok(tx.save(model).await?)
    }

    async fn delete_by_id(&self, tx: &mut MySqlConnection, id: i64) -> Result<u64, LsFileStoreError> {
        Ok(tx.delete_by_id::<FileStoreDataData>(id).await?)
    }
}
