use crate::model::{FileStoreDataData, FileStoreDataDataCodec, FileStoreDataModel};
use crate::repository::db::FileStoreDataRepository;
use ::sqlx::{MySql, Row, Transaction, query};
use c3p0::sqlx::error::into_c3p0_error;
use c3p0::{sqlx::*, *};
use lightspeed_core::error::LsError;

#[derive(Clone)]
pub struct MySqlFileStoreDataRepository {
    repo: SqlxMySqlC3p0Json<u64, FileStoreDataData, FileStoreDataDataCodec>,
}

impl Default for MySqlFileStoreDataRepository {
    fn default() -> Self {
        MySqlFileStoreDataRepository {
            repo: SqlxMySqlC3p0JsonBuilder::new("LS_FILE_STORE_DATA").build_with_codec(FileStoreDataDataCodec {}),
        }
    }
}

impl FileStoreDataRepository for MySqlFileStoreDataRepository {
    type Tx<'a> = Transaction<'a, MySql>;

    async fn exists_by_repository(
        &self,
        tx: &mut Self::Tx<'_>,
        repository: &str,
        file_path: &str,
    ) -> Result<bool, LsError> {
        let sql = "SELECT EXISTS (SELECT 1 FROM LS_FILE_STORE_DATA WHERE (data -> '$.repository') = ? AND (data -> '$.file_path') = ?)";

        let res = query(sql)
            .bind(repository)
            .bind(file_path)
            .fetch_one(tx.as_mut())
            .await
            .and_then(|row| row.try_get(0))
            .map_err(into_c3p0_error)?;
        Ok(res)
    }

    async fn fetch_one_by_id(&self, tx: &mut Self::Tx<'_>, id: u64) -> Result<FileStoreDataModel, LsError> {
        Ok(self.repo.fetch_one_by_id(tx, &id).await?)
    }

    async fn fetch_one_by_repository(
        &self,
        tx: &mut Self::Tx<'_>,
        repository: &str,
        file_path: &str,
    ) -> Result<FileStoreDataModel, LsError> {
        let sql = format!(
            r#"
            {}
            WHERE (data -> '$.repository') = ? AND (data -> '$.file_path') = ?
        "#,
            self.repo.queries().find_base_sql_query
        );

        Ok(self.repo.fetch_one_with_sql(tx, ::sqlx::query(&sql).bind(repository).bind(file_path)).await?)
    }

    async fn fetch_all_by_repository(
        &self,
        tx: &mut Self::Tx<'_>,
        repository: &str,
        offset: usize,
        max: usize,
        sort: &OrderBy,
    ) -> Result<Vec<FileStoreDataModel>, LsError> {
        let sql = format!(
            r#"{}
               WHERE (data -> '$.repository') = ?
                order by id {}
                limit {}
                offset {}
               "#,
            self.repo.queries().find_base_sql_query,
            sort.to_sql(),
            max,
            offset
        );

        Ok(self.repo.fetch_all_with_sql(tx, ::sqlx::query(&sql).bind(repository)).await?)
    }

    async fn save(
        &self,
        tx: &mut Self::Tx<'_>,
        model: NewModel<FileStoreDataData>,
    ) -> Result<FileStoreDataModel, LsError> {
        Ok(self.repo.save(tx, model).await?)
    }

    async fn delete_by_id(&self, tx: &mut Self::Tx<'_>, id: u64) -> Result<u64, LsError> {
        Ok(self.repo.delete_by_id(tx, &id).await?)
    }
}
