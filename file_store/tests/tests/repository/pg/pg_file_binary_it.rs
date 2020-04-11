use crate::{data, test};
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use lightspeed_core::utils::{current_epoch_seconds, new_hyphenated_uuid};
use lightspeed_file_store::repository::FileStoreRepositoryManager;

#[test]
fn should_save_file_in_db() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let file_binary_repo = &data.0.repo_manager.file_store_binary_repo();



        Ok(())
    })
}

