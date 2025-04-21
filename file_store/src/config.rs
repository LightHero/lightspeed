use std::collections::HashMap;
use opendal::Operator;

#[derive(Debug, Clone, Default)]
pub struct FileStoreConfig {
    pub repositories: HashMap<String, Operator>,
}
