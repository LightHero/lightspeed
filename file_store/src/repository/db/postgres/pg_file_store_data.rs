use crate::model::{FileStoreDataData, FileStoreDataModel};
use crate::repository::db::FileStoreDataRepository;
use c3p0::sql::OrderBy;
use c3p0::sqlx::{Postgres, Row, query};
use c3p0::{sqlx::*, *};
use lightspeed_core::error::LsError;

#[derive(Clone)]
pub struct PgFileStoreDataRepository {
}

impl Default for PgFileStoreDataRepository {
    fn default() -> Self {
        PgFileStoreDataRepository {
        }
    }
}

impl FileStoreDataRepository for PgFileStoreDataRepository {
    type DB = Postgres;

    async fn exists_by_repository(
        &self,
        tx: &mut PgConnection,
        repository: &str,
        file_path: &str,
    ) -> Result<bool, LsError> {
        let sql = "SELECT EXISTS (SELECT 1 FROM LS_FILE_STORE_DATA WHERE (data ->> 'repository') = $1 AND (data ->> 'file_path') = $2)";

        let res = query(sql)
            .bind(repository)
            .bind(file_path)
            .fetch_one(tx.as_mut())
            .await
            .and_then(|row| row.try_get(0))?;
        Ok(res)
    }

    async fn fetch_one_by_id(&self, tx: &mut PgConnection, id: u64) -> Result<FileStoreDataModel, LsError> {
        Ok(tx.fetch_one_by_id(id).await?)
    }

    async fn fetch_one_by_repository(
        &self,
        tx: &mut PgConnection,
        repository: &str,
        file_path: &str,
    ) -> Result<FileStoreDataModel, LsError> {
        Ok(FileStoreDataModel::query_with(r#"
            WHERE (data ->> 'repository') = $1 AND (data ->> 'file_path') = $2
        "#)
            .bind(repository).bind(file_path)
            .fetch_one(tx)
            .await?)
    }

    async fn fetch_all_by_repository(
        &self,
        tx: &mut PgConnection,
        repository: &str,
        offset: usize,
        max: usize,
        sort: OrderBy,
    ) -> Result<Vec<FileStoreDataModel>, LsError> {
        Ok(FileStoreDataModel::query_with(&format!(
            r#"
               WHERE (data ->> 'repository') = $1
                order by id {}
                limit {}
                offset {}
               "#,
            sort,
            max,
            offset
        ))
            .bind(repository)
            .fetch_all(tx)
            .await?)
    }

    async fn save(
        &self,
        tx: &mut PgConnection,
        model: NewRecord<FileStoreDataData>,
    ) -> Result<FileStoreDataModel, LsError> {
        Ok(tx.save(model).await?)
    }

    async fn delete_by_id(&self, tx: &mut PgConnection, id: u64) -> Result<u64, LsError> {
        Ok(tx.delete_by_id::<FileStoreDataData>(id).await?)
    }
}
