use opendal::Operator;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub enum RepositoryType {
    Opendal(Arc<Operator>),
    DB
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
}
