use std::{collections::HashMap, sync::Arc};
use opendal::Operator;

#[derive(Debug, Clone, Default)]
pub struct FileStoreConfig {
    pub repositories: HashMap<String, Arc<Operator>>,
}
