use crate::repository::FileStoreBinaryRepository;
use c3p0::pg::*;
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use crate::model::{FileStoreData, FileStoreDataCodec};
use std::path::Path;
use c3p0::pg::tokio_postgres::binary_copy::BinaryCopyInWriter;

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

    async fn read_file<W: tokio::io::AsyncWrite + Unpin + Send>(&self, conn: &mut Self::Conn, file_name: &str, output: &mut W) -> Result<u64, LightSpeedError> {

        match conn {
            PgConnectionAsync::Tx(tx) => {
                /*
                let sql = &format!("SELECT DATA FROM {} WHERE filename = ?", self.table_name);
                let params = &[&file_name];

                let stmt = tx.prepare(sql).await.map_err(into_c3p0_error)?;

                tx.copy_in()

                let row = tx.query_one(&stmt, params)
                    .await
                    .map_err(into_c3p0_error)?;

                row.
                */
            }
        }

        unimplemented!()
    }

    async fn save_file(&self, conn: &mut Self::Conn, source_path: &str, file_name: &str) -> Result<(), LightSpeedError> {

        match conn {
            PgConnectionAsync::Tx(tx) => {
                /*
                let writer = tx.copy_in("COPY foo FROM stdin BINARY").await.unwrap();
                let mut writer = BinaryCopyInWriter::new(writer, &[Type::INT4, Type::TEXT]);
                writer.write(&[&1i32, &"steven"]).await.unwrap();
                writer.write(&[&2i32, &"timothy"]).await.unwrap();
                writer.finish().unwrap();
                */
            }
        }

        unimplemented!()
    }

    async fn delete_by_filename(&self, conn: &mut Self::Conn, file_name: &str) -> Result<(), LightSpeedError> {
        unimplemented!()
    }
}

