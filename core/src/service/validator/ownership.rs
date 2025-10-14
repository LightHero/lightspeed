use c3p0::{DataType};

use crate::error::ErrorDetails;
use crate::service::auth::Owned;

pub const WRONG_OWNER: &str = "WRONG_OWNER";
pub const WRONG_ID: &str = "WRONG_ID";
pub const WRONG_VERSION: &str = "WRONG_VERSION";

pub trait WithIdAndVersion {
    fn get_id(&self) -> u64;
    fn get_version(&self) -> u32;
}

impl<Data: DataType> WithIdAndVersion for c3p0::Record<Data> {
    fn get_id(&self) -> u64 {
        self.id
    }

    fn get_version(&self) -> u32 {
        self.version
    }
}

pub fn validate_ownership<F: Owned, S: Owned>(error_details: &mut ErrorDetails, owner: &F, owned: &S) {
    if owner.get_owner_id() != owned.get_owner_id() {
        error_details.add_detail("owner_id", WRONG_OWNER)
    }
}

pub fn validate_id_and_version<F: WithIdAndVersion, S: WithIdAndVersion>(
    error_details: &mut ErrorDetails,
    first: &F,
    second: &S,
) {
    if first.get_version() != second.get_version() {
        error_details.add_detail("version", WRONG_VERSION)
    }
    if first.get_id() != second.get_id() {
        error_details.add_detail("id", WRONG_ID)
    }
}

pub fn validate_ownership_id_and_version<
    F: Owned + WithIdAndVersion,
    S: Owned + WithIdAndVersion,
>(
    error_details: &mut ErrorDetails,
    first: &F,
    second: &S,
) {
    validate_ownership(error_details, first, second);
    validate_id_and_version(error_details, first, second);
}

#[cfg(test)]
mod tests {

    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::error::{ErrorDetail, ErrorDetails};

    #[derive(Serialize, Deserialize)]
    struct Data(u64);

    impl DataType for Data {
        const TABLE_NAME: &'static str = "";
        type CODEC = Self;
    }

    impl Owned for Data {
        fn get_owner_id(&self) -> u64 {
            self.0
        }
    }

    #[test]
    fn should_validate_ownership_1() {
        // Arrange
        let mut error_details = ErrorDetails::default();

        let first = Data(61);
        let second = Data(61);

        // Act
        validate_ownership(&mut error_details, &first, &second);

        // Assert
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_validate_ownership_2() {
        // Arrange
        let mut error_details = ErrorDetails::default();

        let first =
            c3p0::Record { id: 13, version: 1, data: Data(1000), update_epoch_millis: 0, create_epoch_millis: 0 };
        let second =
            c3p0::Record { id: 12, version: 0, data: Data(1000), update_epoch_millis: 0, create_epoch_millis: 0 };

        // Act
        validate_ownership(&mut error_details, &first, &second);

        // Assert
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_fail_validate_ownership() {
        // Arrange
        let mut error_details = ErrorDetails::default();

        let first =
            c3p0::Record { id: 13, version: 1, data: Data(1001), update_epoch_millis: 0, create_epoch_millis: 0 };
        let second =
            c3p0::Record { id: 12, version: 0, data: Data(1000), update_epoch_millis: 0, create_epoch_millis: 0 };

        // Act
        validate_ownership(&mut error_details, &first, &second);

        // Assert
        assert_eq!(1, error_details.details().len());
        assert_eq!(ErrorDetail::new(WRONG_OWNER, vec![]), error_details.details()["owner_id"][0])
    }

    #[test]
    fn should_validate_id_and_version() {
        // Arrange
        let mut error_details = ErrorDetails::default();

        let first =
            c3p0::Record { id: 12, version: 0, data: Data(1001), update_epoch_millis: 0, create_epoch_millis: 0 };
        let second =
            c3p0::Record { id: 12, version: 0, data: Data(1000), update_epoch_millis: 0, create_epoch_millis: 0 };

        // Act
        validate_id_and_version(&mut error_details, &first, &second);

        // Assert
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_fail_validate_id_and_version() {
        // Arrange
        let mut error_details = ErrorDetails::default();

        let first =
            c3p0::Record { id: 13, version: 1, data: Data(1001), update_epoch_millis: 0, create_epoch_millis: 0 };
        let second =
            c3p0::Record { id: 12, version: 0, data: Data(1000), update_epoch_millis: 0, create_epoch_millis: 0 };

        // Act
        validate_id_and_version(&mut error_details, &first, &second);

        // Assert
        assert_eq!(2, error_details.details().len());
        assert_eq!(ErrorDetail::new(WRONG_ID, vec![]), error_details.details()["id"][0]);
        assert_eq!(ErrorDetail::new(WRONG_VERSION, vec![]), error_details.details()["version"][0]);
    }

    #[test]
    fn should_validate_ownership_id_and_version() {
        // Arrange
        let mut error_details = ErrorDetails::default();

        let first =
            c3p0::Record { id: 12, version: 0, data: Data(1000), update_epoch_millis: 0, create_epoch_millis: 0 };
        let second =
            c3p0::Record { id: 12, version: 0, data: Data(1000), update_epoch_millis: 0, create_epoch_millis: 0 };

        // Act
        validate_ownership_id_and_version(&mut error_details, &first, &second);

        // Assert
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_fail_validate_ownership_id_and_version_if_bad_id() {
        // Arrange
        let mut error_details = ErrorDetails::default();

        let first =
            c3p0::Record { id: 13, version: 0, data: Data(1000), update_epoch_millis: 0, create_epoch_millis: 0 };
        let second =
            c3p0::Record { id: 12, version: 0, data: Data(1000), update_epoch_millis: 0, create_epoch_millis: 0 };

        // Act
        validate_ownership_id_and_version(&mut error_details, &first, &second);

        // Assert
        assert_eq!(1, error_details.details().len());
        assert_eq!(ErrorDetail::new(WRONG_ID, vec![]), error_details.details()["id"][0]);
    }

    #[test]
    fn should_fail_validate_ownership_id_and_version_if_bad_version() {
        // Arrange
        let mut error_details = ErrorDetails::default();

        let first =
            c3p0::Record { id: 12, version: 1, data: Data(1000), update_epoch_millis: 0, create_epoch_millis: 0 };
        let second =
            c3p0::Record { id: 12, version: 0, data: Data(1000), update_epoch_millis: 0, create_epoch_millis: 0 };

        // Act
        validate_ownership_id_and_version(&mut error_details, &first, &second);

        // Assert
        assert_eq!(1, error_details.details().len());
        assert_eq!(ErrorDetail::new(WRONG_VERSION, vec![]), error_details.details()["version"][0]);
    }

    #[test]
    fn should_fail_validate_ownership_id_and_version_if_bad_owner() {
        // Arrange
        let mut error_details = ErrorDetails::default();

        let first =
            c3p0::Record { id: 12, version: 0, data: Data(1001), update_epoch_millis: 0, create_epoch_millis: 0 };
        let second =
            c3p0::Record { id: 12, version: 0, data: Data(1000), update_epoch_millis: 0, create_epoch_millis: 0 };

        // Act
        validate_ownership_id_and_version(&mut error_details, &first, &second);

        // Assert
        assert_eq!(1, error_details.details().len());
        assert_eq!(ErrorDetail::new(WRONG_OWNER, vec![]), error_details.details()["owner_id"][0]);
    }
}
