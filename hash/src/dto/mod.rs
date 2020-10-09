use lightspeed_core::model::language::Language;
use serde::{Deserialize, Serialize};
use typescript_definitions::TypeScriptify;


#[derive(Clone, Serialize, Deserialize, TypeScriptify)]
pub struct ValidationCodeRequestDto {
    pub to_be_validated: String,
    pub code: String,
    pub language: Option<Language>,
    pub validation_code_validity_seconds: i64,
}

#[derive(Clone, Serialize, Deserialize, TypeScriptify)]
pub struct ValidationCodeDataDto {
    pub to_be_validated: String,
    pub created_ts_seconds: i64,
    pub expiration_ts_seconds: i64,
    pub token_hash: String,
}

#[derive(Clone, Serialize, Deserialize, TypeScriptify)]
pub struct VerifyValidationCodeRequestDto {
    pub data: ValidationCodeDataDto,
    pub code: String,
}

#[derive(Clone, Serialize, Deserialize, TypeScriptify)]
pub struct VerifyValidationCodeResponseDto {
    pub to_be_validated: String,
    pub code_valid: bool,
}
