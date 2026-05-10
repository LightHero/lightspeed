use opendal::Operator;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub enum RepositoryType {
    Opendal(Arc<Operator>),
    DB,
}

impl From<Arc<Operator>> for RepositoryType {
    fn from(value: Arc<Operator>) -> Self {
        RepositoryType::Opendal(value)
    }
}

impl From<Operator> for RepositoryType {
    fn from(value: Operator) -> Self {
        RepositoryType::Opendal(Arc::new(value))
    }
}

#[derive(Debug, Clone, Default)]
pub struct FileStoreConfig {
    pub repositories: HashMap<String, RepositoryType>,

    /// Maximum byte size accepted when saving a file, regardless of the
    /// destination repository type. `None` disables the cap. Oversized
    /// writes are rejected with `ErrorCodes::PAYLOAD_TOO_LARGE` before
    /// the bytes are materialized: this both guards the in-memory buffer
    /// that backs BYTEA / BLOB columns for DB repositories and enforces
    /// per-file quota for OpenDal repositories.
    pub save_max_size_bytes: Option<usize>,
}
