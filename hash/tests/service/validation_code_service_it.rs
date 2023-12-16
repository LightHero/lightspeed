use crate::init;
use chrono::prelude::*;
use lightspeed_core::error::LsError;
use lightspeed_core::model::language::Language;
use lightspeed_hash::dto::{ValidationCodeRequestDto, VerifyValidationCodeRequestDto};

#[tokio::test]
async fn should_generate_validation_code() -> Result<(), LsError> {
    // Arrange
    let hash_module = init().await;
    let validation_code_service = &hash_module.validation_code_service;

    let validation_code_validity_seconds = 100;

    let validation_code_request = ValidationCodeRequestDto {
        to_be_validated: "123456789".to_owned(),
        language: Some(Language::It),
        code: format!("{}", Utc::now().timestamp_millis()),
        validation_code_validity_seconds,
    };

    // Act
    let validation_code_response = validation_code_service.generate_validation_code(validation_code_request.clone())?;
    assert_eq!(validation_code_request.to_be_validated, validation_code_response.to_be_validated);
    assert_eq!(
        validation_code_response.created_ts_seconds + validation_code_validity_seconds,
        validation_code_response.expiration_ts_seconds
    );

    // Verify valid code
    let verify_code_request =
        VerifyValidationCodeRequestDto { data: validation_code_response, code: validation_code_request.code };

    let verify_code_response = validation_code_service.verify_validation_code(verify_code_request.clone())?;
    assert!(verify_code_response.code_valid);
    assert_eq!(verify_code_request.data.to_be_validated, verify_code_response.to_be_validated);

    // Use bad code
    let mut bad_verify_code = verify_code_request.clone();
    bad_verify_code.code = "abced".to_owned();
    assert!(!validation_code_service.verify_validation_code(bad_verify_code).unwrap().code_valid);

    // tampered to_be_validated data
    let mut bad_verify_code = verify_code_request.clone();
    bad_verify_code.data.to_be_validated = "2233223322".to_owned();
    assert!(!validation_code_service.verify_validation_code(bad_verify_code).unwrap().code_valid);

    // tampered created_ts_seconds
    let mut bad_verify_code = verify_code_request.clone();
    bad_verify_code.data.created_ts_seconds = verify_code_request.data.created_ts_seconds + 1;
    assert!(!validation_code_service.verify_validation_code(bad_verify_code).unwrap().code_valid);

    // tampered expiration_ts_seconds
    let mut bad_verify_code = verify_code_request.clone();
    bad_verify_code.data.expiration_ts_seconds = verify_code_request.data.expiration_ts_seconds + 1;
    assert!(!validation_code_service.verify_validation_code(bad_verify_code).unwrap().code_valid);

    // tampered token_hash number
    let mut bad_verify_code = verify_code_request.clone();
    bad_verify_code.data.token_hash = format!("{}1", verify_code_request.data.token_hash);
    assert!(!validation_code_service.verify_validation_code(bad_verify_code).unwrap().code_valid);

    Ok(())
}
