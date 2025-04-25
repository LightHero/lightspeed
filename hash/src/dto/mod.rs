use lightspeed_core::model::language::Language;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ValidationCodeRequestDto<Data> {
    pub to_be_validated: Data,
    pub code: String,
    pub language: Option<Language>,
    pub validation_code_validity_seconds: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ValidationCodeDataDto<Data> {
    pub to_be_validated: Data,
    pub created_ts_seconds: i64,
    pub expiration_ts_seconds: i64,
    pub token_hash: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VerifyValidationCodeRequestDto<Data> {
    pub data: ValidationCodeDataDto<Data>,
    pub code: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VerifyValidationCodeResponseDto<Data> {
    pub to_be_validated: Data,
    pub code_valid: bool,
}
