use c3p0::Model;
use lightspeed_core::error::{ErrorDetails, LightSpeedError};
use lightspeed_core::service::validator::number::validate_number_ge;
use lightspeed_core::service::validator::Validable;
use serde::{Deserialize, Serialize};

pub type ProjectModel = Model<ProjectData>;

#[derive(Clone, Serialize, Deserialize)]
pub struct ProjectData {
    pub name: String,
}

impl Validable for &ProjectData {
    fn validate<E: ErrorDetails>(&self, error_details: &mut E) -> Result<(), LightSpeedError> {
        validate_number_ge(error_details, "name", 3, self.name.len());
        Ok(())
    }
}

#[cfg(test)]
pub mod test {

    use super::*;
    use lightspeed_core::error::ErrorDetail;
    use lightspeed_core::service::validator::number::MUST_BE_GREATER_OR_EQUAL;
    use lightspeed_core::service::validator::Validator;

    #[test]
    pub fn validation_should_fail_if_name_too_short() {
        let project_data = ProjectData {
            name: "".to_owned(),
        };

        // Act
        let result = Validator::validate(&project_data);

        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert_eq!(details.details().borrow().len(), 1);
                assert_eq!(
                    details.details().borrow().get("name"),
                    Some(&vec![ErrorDetail::new(
                        MUST_BE_GREATER_OR_EQUAL,
                        vec!["3".to_owned()]
                    )])
                );
            }
            _ => assert!(false),
        }
    }

    #[test]
    pub fn should_validate() {
        let project_data = ProjectData {
            name: "good name".to_owned(),
        };

        // Act
        let result = Validator::validate(&project_data);

        // Assert
        assert!(result.is_ok());
    }
}
