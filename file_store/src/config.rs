use opendal::Operator;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone, Default)]
pub struct FileStoreConfig {
    pub repositories: HashMap<String, Arc<Operator>>,
}
