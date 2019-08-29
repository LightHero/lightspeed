use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum Language {
    En,
    It,
    Fr,
    De,
    Es,
}
