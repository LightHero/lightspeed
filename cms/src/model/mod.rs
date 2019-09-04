
pub struct Schema {
    pub name: String,
    pub fields: Vec<SchemaField>
}

pub struct SchemaField {
    pub label: String,
    pub description: String,
    pub category: SchemaFieldCategory,
    pub field_type: SchemaFieldType
}

pub enum SchemaFieldCategory {
    NotRequired,
    Required,
    Localizable
}

pub enum SchemaFieldType {
    Number{
        min: Option<u64>,
        max: Option<u64>,
        default: Option<u64>,
    },
    String{
        min_length: Option<String>,
        max_length: Option<String>,
        default: Option<String>,
    },
    Boolean{
        default: Option<bool>,
    }
}

pub struct Content {
    pub schema_name: String,
    pub fields: Vec<ContentField>
}

pub struct ContentField {
    pub label: String,
    pub field_type: ContentFieldType
}

pub enum ContentFieldType {
    Number(Option<u64>),
    String(Option<String>),
    Boolean(Option<bool>)
}