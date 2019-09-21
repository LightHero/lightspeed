use c3p0::Model;
use serde::{Deserialize, Serialize};

pub type SchemaContentMappingModel = Model<SchemaContentMappingData>;

#[derive(Clone, Serialize, Deserialize)]
pub struct SchemaContentMappingData {
    pub project_name: String,
    pub schema_name: String,
    pub content_table: String,
}
