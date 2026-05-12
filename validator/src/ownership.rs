use crate::error::{ErrorDetails, ValidationError};

pub trait Owned {
    fn get_owner_id(&self) -> i64;
}

pub trait WithIdAndVersion {
    fn get_id(&self) -> i64;
    fn get_version(&self) -> i64;
}

pub fn validate_ownership<F: Owned, S: Owned>(error_details: &mut ErrorDetails, owner: &F, owned: &S) {
    if owner.get_owner_id() != owned.get_owner_id() {
        error_details.add_detail("owner_id", ValidationError::WrongOwner)
    }
}

pub fn validate_id_and_version<F: WithIdAndVersion, S: WithIdAndVersion>(
    error_details: &mut ErrorDetails,
    first: &F,
    second: &S,
) {
    if first.get_version() != second.get_version() {
        error_details.add_detail("version", ValidationError::WrongVersion)
    }
    if first.get_id() != second.get_id() {
        error_details.add_detail("id", ValidationError::WrongId)
    }
}

pub fn validate_ownership_id_and_version<F: Owned + WithIdAndVersion, S: Owned + WithIdAndVersion>(
    error_details: &mut ErrorDetails,
    first: &F,
    second: &S,
) {
    validate_ownership(error_details, first, second);
    validate_id_and_version(error_details, first, second);
}

#[cfg(test)]
mod tests {

    use c3p0::DataType;
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Serialize, Deserialize)]
    struct Data(i64);

    impl DataType for Data {
        const TABLE_NAME: &'static str = "";
        type CODEC = Self;
    }

    impl Owned for Data {
        fn get_owner_id(&self) -> i64 {
            self.0
        }
    }

    impl<D: Owned + DataType> Owned for c3p0::Record<D> {
        fn get_owner_id(&self) -> i64 {
            self.data.get_owner_id()
        }
    }

    impl<D: DataType> WithIdAndVersion for c3p0::Record<D> {
        fn get_id(&self) -> i64 {
            self.id
        }
        fn get_version(&self) -> i64 {
            self.version
        }
    }

    #[test]
    fn should_validate_ownership_1() {
        let mut error_details = ErrorDetails::default();
        let first = Data(61);
        let second = Data(61);
        validate_ownership(&mut error_details, &first, &second);
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_validate_ownership_2() {
        let mut error_details = ErrorDetails::default();
        let first = c3p0::Record {
            id: 13,
            version: 1,
            data: Data(1000),
            update_time: Default::default(),
            create_time: Default::default(),
        };
        let second = c3p0::Record {
            id: 12,
            version: 0,
            data: Data(1000),
            update_time: Default::default(),
            create_time: Default::default(),
        };
        validate_ownership(&mut error_details, &first, &second);
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_fail_validate_ownership() {
        let mut error_details = ErrorDetails::default();
        let first = c3p0::Record {
            id: 13,
            version: 1,
            data: Data(1001),
            update_time: Default::default(),
            create_time: Default::default(),
        };
        let second = c3p0::Record {
            id: 12,
            version: 0,
            data: Data(1000),
            update_time: Default::default(),
            create_time: Default::default(),
        };
        validate_ownership(&mut error_details, &first, &second);
        assert_eq!(1, error_details.details().len());
        assert_eq!(ValidationError::WrongOwner, error_details.details()["owner_id"][0])
    }

    #[test]
    fn should_validate_id_and_version() {
        let mut error_details = ErrorDetails::default();
        let first = c3p0::Record {
            id: 12,
            version: 0,
            data: Data(1001),
            update_time: Default::default(),
            create_time: Default::default(),
        };
        let second = c3p0::Record {
            id: 12,
            version: 0,
            data: Data(1000),
            update_time: Default::default(),
            create_time: Default::default(),
        };
        validate_id_and_version(&mut error_details, &first, &second);
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_fail_validate_id_and_version() {
        let mut error_details = ErrorDetails::default();
        let first = c3p0::Record {
            id: 13,
            version: 1,
            data: Data(1001),
            update_time: Default::default(),
            create_time: Default::default(),
        };
        let second = c3p0::Record {
            id: 12,
            version: 0,
            data: Data(1000),
            update_time: Default::default(),
            create_time: Default::default(),
        };
        validate_id_and_version(&mut error_details, &first, &second);
        assert_eq!(2, error_details.details().len());
        assert_eq!(ValidationError::WrongId, error_details.details()["id"][0]);
        assert_eq!(ValidationError::WrongVersion, error_details.details()["version"][0]);
    }

    #[test]
    fn should_validate_ownership_id_and_version() {
        let mut error_details = ErrorDetails::default();
        let first = c3p0::Record {
            id: 12,
            version: 0,
            data: Data(1000),
            update_time: Default::default(),
            create_time: Default::default(),
        };
        let second = c3p0::Record {
            id: 12,
            version: 0,
            data: Data(1000),
            update_time: Default::default(),
            create_time: Default::default(),
        };
        validate_ownership_id_and_version(&mut error_details, &first, &second);
        assert!(error_details.details().is_empty())
    }

    #[test]
    fn should_fail_validate_ownership_id_and_version_if_bad_id() {
        let mut error_details = ErrorDetails::default();
        let first = c3p0::Record {
            id: 13,
            version: 0,
            data: Data(1000),
            update_time: Default::default(),
            create_time: Default::default(),
        };
        let second = c3p0::Record {
            id: 12,
            version: 0,
            data: Data(1000),
            update_time: Default::default(),
            create_time: Default::default(),
        };
        validate_ownership_id_and_version(&mut error_details, &first, &second);
        assert_eq!(1, error_details.details().len());
        assert_eq!(ValidationError::WrongId, error_details.details()["id"][0]);
    }

    #[test]
    fn should_fail_validate_ownership_id_and_version_if_bad_version() {
        let mut error_details = ErrorDetails::default();
        let first = c3p0::Record {
            id: 12,
            version: 1,
            data: Data(1000),
            update_time: Default::default(),
            create_time: Default::default(),
        };
        let second = c3p0::Record {
            id: 12,
            version: 0,
            data: Data(1000),
            update_time: Default::default(),
            create_time: Default::default(),
        };
        validate_ownership_id_and_version(&mut error_details, &first, &second);
        assert_eq!(1, error_details.details().len());
        assert_eq!(ValidationError::WrongVersion, error_details.details()["version"][0]);
    }

    #[test]
    fn should_fail_validate_ownership_id_and_version_if_bad_owner() {
        let mut error_details = ErrorDetails::default();
        let first = c3p0::Record {
            id: 12,
            version: 0,
            data: Data(1001),
            update_time: Default::default(),
            create_time: Default::default(),
        };
        let second = c3p0::Record {
            id: 12,
            version: 0,
            data: Data(1000),
            update_time: Default::default(),
            create_time: Default::default(),
        };
        validate_ownership_id_and_version(&mut error_details, &first, &second);
        assert_eq!(1, error_details.details().len());
        assert_eq!(ValidationError::WrongOwner, error_details.details()["owner_id"][0]);
    }
}
