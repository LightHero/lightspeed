use crate::model::content::Content;
use crate::model::schema::Schema;
use lightspeed_core::error::{ErrorDetails, LightSpeedError};
use lightspeed_core::service::validator::Validator;

pub struct ContentService {}

impl ContentService {
    pub fn validate_content(_schema: &Schema, _content: &Content) -> Result<(), LightSpeedError> {
        Validator::validate(|_error_details: &ErrorDetails| Ok(()))
    }
}

#[cfg(test)]
mod test {}
