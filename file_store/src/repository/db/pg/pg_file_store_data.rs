use crate::repository::db::{FileStoreDataRepository};
use c3p0::postgres::*;
use c3p0::*;
use lightspeed_core::error::{ErrorCodes, LightSpeedError};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use crate::model::{FileStoreDataData, FileStoreDataDataCodec};
use std::ops::Deref;

#[derive(Clone)]
pub struct PgFileStoreDataRepository {
    repo: PgC3p0Json<FileStoreDataData, FileStoreDataDataCodec>,
}

impl Deref for PgFileStoreDataRepository {
    type Target = PgC3p0Json<FileStoreDataData, FileStoreDataDataCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl Default for PgFileStoreDataRepository {
    fn default() -> Self {
        PgFileStoreDataRepository {
            repo: C3p0JsonBuilder::new("LS_FILE_STORE_DATA").build_with_codec(FileStoreDataDataCodec {}),
        }
    }
}

#[async_trait::async_trait]
impl FileStoreDataRepository for PgFileStoreDataRepository {
    type Conn = PgConnection;

}
