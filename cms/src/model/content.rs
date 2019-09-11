pub struct Content {
    pub fields: Vec<ContentField>,
    pub created_ms: i64,
    pub updated_ms: i64,
}

pub struct ContentField {
    pub label: String,
    pub value: ContentFieldValue,
}

pub enum ContentFieldValue {
    Number(Option<usize>),
    String(Option<String>),
    Boolean(Option<bool>),
}
