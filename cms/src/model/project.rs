use c3p0::Model;
use lightspeed_core::error::{ErrorDetails, LsError};
use lightspeed_core::service::validator::Validable;
use lightspeed_core::service::validator::order::validate_ge;
use serde::{Deserialize, Serialize};

pub type ProjectModel = Model<u64, ProjectData>;

#[derive(Clone, Serialize, Deserialize)]
pub struct ProjectData {
    pub name: String,
}

impl Validable for ProjectData {
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LsError> {
        validate_ge(error_details, "name", 3, self.name.len());
        Ok(())
    }
}

#[cfg(test)]
pub mod test {

    use super::*;
    use lightspeed_core::error::ErrorDetail;
    use lightspeed_core::service::validator::Validator;
    use lightspeed_core::service::validator::order::MUST_BE_GREATER_OR_EQUAL;

    #[test]
    pub fn validation_should_fail_if_name_too_short() {
        let project_data = ProjectData { name: "".to_owned() };

        // Act
        let result = Validator::validate(&project_data);

        match result {
            Err(LsError::ValidationError { details }) => {
                assert_eq!(details.details.len(), 1);
                assert_eq!(
                    details.details.get("name"),
                    Some(&vec![ErrorDetail::new(MUST_BE_GREATER_OR_EQUAL, vec!["3".to_owned()])])
                );
            }
            _ => panic!(),
        }
    }

    #[test]
    pub fn should_validate() {
        let project_data = ProjectData { name: "good name".to_owned() };

        // Act
        let result = Validator::validate(&project_data);

        // Assert
        assert!(result.is_ok());
    }
}
