use crate::{data, test};
use lightspeed_core::error::LightSpeedError;

#[test]
fn should_save_file_in_db() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_store_module = &data.0.file_store_service;

        Ok(())
    })
}
