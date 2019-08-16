use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Language {
    En,
    It,
    Fr,
    De,
    Es,
}
