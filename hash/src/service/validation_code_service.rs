use crate::dto::{ValidationCodeRequestDto, ValidationCodeDataDto, VerifyValidationCodeRequestDto, VerifyValidationCodeResponseDto};
use lightspeed_core::error::LightSpeedError;
use lightspeed_core::service::jwt::{JwtService, JWT};
use lightspeed_core::utils::current_epoch_seconds;
use log::*;
use serde::Serialize;
use std::sync::Arc;
use crate::service::hash_service::HashService;

#[derive(Clone)]
pub struct ValidationCodeService {
    jwt_service: Arc<JwtService>,
    hash_service: Arc<HashService>,
}

#[derive(Clone, Serialize)]
struct ValidationCodeData<'a> {
    to_be_validated: &'a str,
    code: &'a str,
    created_ts_seconds: i64,
    expiration_ts_seconds: i64,
}

impl ValidationCodeService {
    pub fn new(
        hash_service: Arc<HashService>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            jwt_service,
            hash_service,
        }
    }

    pub fn random_numeric_code(&self) -> String {
        use rand::Rng;
        let number: u32 = rand::thread_rng().gen_range(0, 1_000_000);
        format!("{:06}", number)
    }

    pub fn generate_validation_code(&self, request: ValidationCodeRequestDto) -> Result<ValidationCodeDataDto, LightSpeedError> {
        info!("Generate validation code");

        let created_ts_seconds = current_epoch_seconds();
        let expiration_ts_seconds = created_ts_seconds + request.validation_code_validity_seconds;

        let token_hash = self.hash(ValidationCodeData {
            expiration_ts_seconds,
            created_ts_seconds,
            code: &request.code,
            to_be_validated: &request.to_be_validated,
        })?;

        Ok(ValidationCodeDataDto { to_be_validated: request.to_be_validated.clone(), expiration_ts_seconds, created_ts_seconds, token_hash })

    }

    pub fn verify_validation_code(&self, request: VerifyValidationCodeRequestDto) -> Result<VerifyValidationCodeResponseDto, LightSpeedError> {
        debug!("Verify number {} with code {}", request.data.to_be_validated, request.code);
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

    fn hash(&self, data: ValidationCodeData) -> Result<String, LightSpeedError> {
        let jwt = JWT { iat: data.created_ts_seconds, exp: data.expiration_ts_seconds, sub: "".to_owned(), payload: data };
        let token = self.jwt_service.generate_from_token(&jwt)?;
        Ok(self.hash_service.hash(&token))
    }
}
