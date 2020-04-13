use crate::{data, test};
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use lightspeed_core::utils::{current_epoch_seconds, new_hyphenated_uuid};

#[test]
fn should_save_file_in_db() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store_module = &data.0.file_store_service;

        Ok(())
    })
}
