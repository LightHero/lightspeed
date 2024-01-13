use crate::dto::{
    ValidationCodeDataDto, ValidationCodeRequestDto, VerifyValidationCodeRequestDto, VerifyValidationCodeResponseDto,
};
use crate::service::hash_service::LsHashService;
use lightspeed_core::error::LsError;
use lightspeed_core::service::jwt::{LsJwtService, JWT};
use lightspeed_core::utils::current_epoch_seconds;
use log::*;
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct LsValidationCodeService {
    jwt_service: Arc<LsJwtService>,
    hash_service: Arc<LsHashService>,
}

#[derive(Clone, Serialize)]
struct ValidationCodeData<'a, Data: Serialize> {
    to_be_validated: &'a Data,
    code: &'a str,
    created_ts_seconds: i64,
    expiration_ts_seconds: i64,
}

impl LsValidationCodeService {
    pub fn new(hash_service: Arc<LsHashService>, jwt_service: Arc<LsJwtService>) -> Self {
        Self { jwt_service, hash_service }
    }

    pub fn generate_validation_code<Data: Serialize>(
        &self,
        request: ValidationCodeRequestDto<Data>,
    ) -> Result<ValidationCodeDataDto<Data>, LsError> {
        info!("Generate validation code");

        let created_ts_seconds = current_epoch_seconds();
        let expiration_ts_seconds = created_ts_seconds + request.validation_code_validity_seconds;

        let token_hash = self.hash(ValidationCodeData {
            expiration_ts_seconds,
            created_ts_seconds,
            code: &request.code,
            to_be_validated: &request.to_be_validated,
        })?;

        Ok(ValidationCodeDataDto {
            to_be_validated: request.to_be_validated,
            expiration_ts_seconds,
            created_ts_seconds,
            token_hash,
        })
    }

    pub fn verify_validation_code<Data: Serialize>(
        &self,
        request: VerifyValidationCodeRequestDto<Data>,
    ) -> Result<VerifyValidationCodeResponseDto<Data>, LsError> {
        debug!("Verify code {}", request.code);
        let calculated_token_hash = self.hash(ValidationCodeData {
            expiration_ts_seconds: request.data.expiration_ts_seconds,
            created_ts_seconds: request.data.created_ts_seconds,
            code: &request.code,
            to_be_validated: &request.data.to_be_validated,
        })?;
        Ok(VerifyValidationCodeResponseDto {
            to_be_validated: request.data.to_be_validated,
            code_valid: calculated_token_hash.eq(&request.data.token_hash),
        })
    }

    fn hash<Data: Serialize>(&self, data: ValidationCodeData<Data>) -> Result<String, LsError> {
        let jwt =
            JWT { iat: data.created_ts_seconds, exp: data.expiration_ts_seconds, sub: "".to_owned(), payload: data };
        let token = self.jwt_service.generate_from_token(&jwt)?;
        Ok(self.hash_service.hash(&token))
    }
}
