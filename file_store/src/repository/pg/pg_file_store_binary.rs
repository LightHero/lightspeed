use crate::repository::FileStoreBinaryRepository;
use c3p0::pg::*;
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use crate::model::{FileStoreData, FileStoreDataCodec};
use std::path::Path;
use c3p0::pg::tokio_postgres::binary_copy::BinaryCopyInWriter;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use crate::dto::FileData;

#[derive(Clone)]
pub struct PgFileStoreBinaryRepository {
    table_name: &'static str,
}

impl Default for PgFileStoreBinaryRepository {
    fn default() -> Self {
        PgFileStoreBinaryRepository {
            table_name: "LS_FILE_STORE_BINARY",
        }
    }
}

#[async_trait::async_trait]
impl FileStoreBinaryRepository for PgFileStoreBinaryRepository {
    type Conn = PgConnectionAsync;

    async fn read_file(&self, conn: &mut Self::Conn, file_name: &str) -> Result<FileData, LightSpeedError> {

        match conn {
            PgConnectionAsync::Tx(tx) => {

                let sql = &format!("SELECT DATA FROM {} WHERE filename = $1", self.table_name);

                let stmt = tx.prepare(sql).await.map_err(into_c3p0_error)?;
                let row = tx.query_one(&stmt, &[&file_name])
                    .await
                    .map_err(into_c3p0_error)?;

                let content: Vec<u8> = row.try_get(0).map_err(into_c3p0_error)?;
                Ok(FileData::InMemory {content})
            }
        }

    }

    async fn save_file(&self, conn: &mut Self::Conn, source_path: &str, file_name: &str) -> Result<(), LightSpeedError> {

        match conn {
            PgConnectionAsync::Tx(tx) => {
                let mut file = File::open(source_path).await.map_err(|err| LightSpeedError::BadRequest {
                    message: format!(
                        "PgFileStoreBinaryRepository - Cannot open file [{}]. Err: {}",
                        file_name,
                        err
                    ),
                })?;
                let mut contents = vec![];
                file.read_to_end(&mut contents).await.map_err(|err| LightSpeedError::BadRequest {
                    message: format!(
                        "PgFileStoreBinaryRepository - Cannot read file [{}]. Err: {}",
                        file_name,
                        err
                    ),
                })?;

                let sql = &format!("INSERT INTO {} (filename, data) VALUES ($1, $2)", self.table_name);

                let stmt = tx.prepare(sql).await.map_err(into_c3p0_error)?;
                Ok(tx.execute(&stmt, &[&file_name, &contents]).await.map_err(into_c3p0_error).map(|_| ())?)
            }
        }
    }

    async fn delete_by_filename(&self, conn: &mut Self::Conn, file_name: &str) -> Result<(), LightSpeedError> {
        unimplemented!()
    }
}

