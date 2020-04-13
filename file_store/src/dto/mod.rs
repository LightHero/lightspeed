
pub enum FileData {
    FromFs{ file_path: String},
    InMemory{content: Vec<u8>}
}