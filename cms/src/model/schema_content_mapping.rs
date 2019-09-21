use c3p0::Model;
use serde::{Deserialize, Serialize};

pub type SchemaContentMappingModel = Model<SchemaContentMappingData>;

#[derive(Clone, Serialize, Deserialize)]
pub struct SchemaContentMappingData {
    pub schema_id: i64,
    pub content_table: String,
}
