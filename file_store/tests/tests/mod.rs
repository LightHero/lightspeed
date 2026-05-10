use futures::StreamExt;
use lightspeed_core::error::LsError;
use lightspeed_file_store::config::{FileStoreConfig, RepositoryType};
use lightspeed_file_store::model::BinaryContent;
use opendal::{Operator, services::Fs};

pub mod repository;
pub mod service;

/// Test-only helper that drains a `BinaryContent` into a contiguous `Vec<u8>`
/// for byte-level assertions. The name signals what's happening: the stream
/// (if any) is fully buffered. Production code must NOT do this — it has to
/// route streams through their native streaming consumer instead.
pub async fn collect_bytes(content: BinaryContent<'_>) -> Result<Vec<u8>, LsError> {
    match content {
        BinaryContent::InMemory { content } => Ok(content.into_owned()),
        BinaryContent::OpenDal { operator, path } => {
            Ok(operator.read(&path).await.map_err(|err| LsError::InternalServerError {
                message: format!("collect_bytes - opendal read [{path}]: {err:?}"),
            })?.to_vec())
        }
        BinaryContent::Stream { stream } => {
            let mut s = stream.into_inner();
            let mut buf: Vec<u8> = Vec::new();
            while let Some(chunk) = s.next().await {
                buf.extend_from_slice(&chunk?);
            }
            Ok(buf)
        }
    }
}

pub fn get_config() -> FileStoreConfig {
    let mut file_store_config = FileStoreConfig::default();
    file_store_config
        .repositories
        .insert("FS_ONE".to_owned(), Operator::new(Fs::default().root("../target/repo_one")).unwrap().finish().into());
    file_store_config
        .repositories
        .insert("FS_TWO".to_owned(), Operator::new(Fs::default().root("../target/repo_two")).unwrap().finish().into());
    file_store_config.repositories.insert("DB_ONE".to_owned(), RepositoryType::DB);
    file_store_config.repositories.insert("DB_TWO".to_owned(), RepositoryType::DB);

    file_store_config
}
