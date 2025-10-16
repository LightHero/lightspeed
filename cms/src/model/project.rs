use c3p0::{Codec, DataType, Record};
use lightspeed_core::error::{ErrorDetails, LsError};
use lightspeed_core::service::validator::Validable;
use lightspeed_core::service::validator::order::validate_ge;
use serde::{Deserialize, Serialize};

pub type ProjectModel = Record<ProjectData>;

#[derive(Clone, Serialize, Deserialize)]
pub struct ProjectData {
    pub name: String,
}

impl DataType for ProjectData {
    const TABLE_NAME: &'static str = "LS_CMS_PROJECT";
    type CODEC = ProjectDataCodec;
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ProjectDataCodec {
    V1(ProjectData),
}

impl Codec<ProjectData> for ProjectDataCodec {
    fn encode(data: ProjectData) -> Self {
        ProjectDataCodec::V1(data)
    }

    fn decode(data: Self) -> ProjectData {
        match data {
            ProjectDataCodec::V1(data) => data,
        }
    }
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
