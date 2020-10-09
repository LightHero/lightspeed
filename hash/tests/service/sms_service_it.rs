use crate::init;
use chrono::prelude::*;
use lightspeed_core::error::LightSpeedError;
use lightspeed_core::model::language::Language;
use oregold_sms::dto::{SendSmsCodeRequestDto, VerifySmsCodeRequestDto};

#[tokio::test]
async fn should_send_and_verify_sms_code() -> Result<(), LightSpeedError> {
    // Arrange
    let sms_module = init().await;
    let sms_service = &sms_module.sms_service;

    let sms_request = SendSmsCodeRequestDto {
        to: "123456789".to_owned(),
        language: Some(Language::IT),
        code: format!("{}", Utc::now().timestamp_millis()),
    };

    // Act
    let sms_code_response = sms_service.send_code_now(sms_request.clone()).await?;
    assert_eq!(sms_request.to, sms_code_response.to);
    assert_eq!(
        sms_code_response.created_ts_seconds + (sms_module.sms_config.sms_verification_code_validity_seconds as i64),
        sms_code_response.expiration_ts_seconds
    );

    // Verify valid code
    let sms_verify_code = VerifySmsCodeRequestDto { sms_code_data: sms_code_response.clone(), code: sms_request.code.clone() };

    let sms_verify_code_response = sms_service.verify_code(sms_verify_code.clone())?;
    assert!(sms_verify_code_response.code_valid);
    assert_eq!(sms_verify_code.sms_code_data.to, sms_verify_code_response.to);

    // Use bad code
    let mut bad_sms_verify_code = sms_verify_code.clone();
    bad_sms_verify_code.code = "abced".to_owned();
    assert!(!sms_service.verify_code(bad_sms_verify_code.clone()).unwrap().code_valid);

    // tampered phone number
    let mut bad_sms_verify_code = sms_verify_code.clone();
    bad_sms_verify_code.sms_code_data.to = "2233223322".to_owned();
    assert!(!sms_service.verify_code(bad_sms_verify_code.clone()).unwrap().code_valid);

    // tampered created_ts_seconds
    let mut bad_sms_verify_code = sms_verify_code.clone();
    bad_sms_verify_code.sms_code_data.created_ts_seconds = sms_verify_code.sms_code_data.created_ts_seconds + 1;
    assert!(!sms_service.verify_code(bad_sms_verify_code.clone()).unwrap().code_valid);

    // tampered expiration_ts_seconds
    let mut bad_sms_verify_code = sms_verify_code.clone();
    bad_sms_verify_code.sms_code_data.expiration_ts_seconds = sms_verify_code.sms_code_data.expiration_ts_seconds + 1;
    assert!(!sms_service.verify_code(bad_sms_verify_code.clone()).unwrap().code_valid);

    // tampered token_hash number
    let mut bad_sms_verify_code = sms_verify_code.clone();
    bad_sms_verify_code.sms_code_data.token_hash = format!("{}1", sms_verify_code.sms_code_data.token_hash);
    assert!(!sms_service.verify_code(bad_sms_verify_code.clone()).unwrap().code_valid);

    Ok(())
}
