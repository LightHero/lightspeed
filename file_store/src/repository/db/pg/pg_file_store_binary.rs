use crate::model::BinaryContent;
use crate::repository::db::DBFileStoreBinaryRepository;
use c3p0::postgres::*;
use lightspeed_core::error::{ErrorCodes, LightSpeedError};
use std::borrow::Cow;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

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
impl DBFileStoreBinaryRepository for PgFileStoreBinaryRepository {
    type Conn = PgConnection;

    async fn read_file<'a>(
        &self,
        conn: &mut Self::Conn,
        repository_name: &str,
        file_path: &str,
    ) -> Result<BinaryContent<'a>, LightSpeedError> {
        let sql = &format!(
            "SELECT DATA FROM {} WHERE repository = $1 AND filepath = $2",
            self.table_name
        );

        let content = conn
            .fetch_one(&sql, &[&repository_name, &file_path], |row| {
                let content: Vec<u8> = row.try_get(0).map_err(into_c3p0_error)?;
                Ok(content)
            })
            .await?;
        Ok(BinaryContent::InMemory {
            content: Cow::Owned(content),
        })
    }

    async fn save_file<'a>(
        &self,
        conn: &mut Self::Conn,
        repository_name: &str,
        file_path: &str,
        content: &'a BinaryContent<'a>,
    ) -> Result<u64, LightSpeedError> {
        let binary_content = match content {
            BinaryContent::InMemory { content } => Cow::Borrowed(content),
            BinaryContent::FromFs { file_path } => {
                let mut file =
                    File::open(file_path)
                        .await
                        .map_err(|err| LightSpeedError::BadRequest {
                            message: format!(
                                "PgFileStoreBinaryRepository - Cannot open file [{}]. Err: {}",
                                file_path.display(),
                                err
                            ),
                            code: ErrorCodes::IO_ERROR,
                        })?;
                let mut contents = vec![];
                file.read_to_end(&mut contents).await.map_err(|err| {
                    LightSpeedError::BadRequest {
                        message: format!(
                            "PgFileStoreBinaryRepository - Cannot read file [{}]. Err: {}",
                            file_path.display(),
                            err
                        ),
                        code: ErrorCodes::IO_ERROR,
                    }
                })?;
                Cow::Owned(Cow::Owned(contents))
            }
        };

        let sql = &format!(
            "INSERT INTO {} (repository, filepath, data) VALUES ($1, $2, $3)",
            self.table_name
        );

        Ok(conn
            .execute(
                &sql,
                &[
                    &repository_name,
                    &file_path,
                    &binary_content.as_ref().as_ref(),
                ],
            )
            .await?)
    }

    async fn delete_file(
        &self,
        conn: &mut Self::Conn,
        repository_name: &str,
        file_path: &str,
    ) -> Result<u64, LightSpeedError> {
        let sql = &format!(
            "DELETE FROM {} WHERE repository = $1 AND filepath = $2",
            self.table_name
        );
        Ok(conn.execute(&sql, &[&repository_name, &file_path]).await?)
    }
}
