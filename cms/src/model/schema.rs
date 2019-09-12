use crate::model::content::{Content, ContentField, ContentFieldValue};
use lightspeed_core::error::{ErrorDetails, LightSpeedError};
use lightspeed_core::service::validator::number::{validate_number_ge, validate_number_le};
use lightspeed_core::service::validator::Validable;

pub const SLUG_VALIDATION_REGEX: &str = "^[a-z0-9]+(?:-[a-z0-9]+)*$";

pub struct Schema {
    pub name: String,
    pub fields: Vec<SchemaField>,
    pub created_ms: i64,
    pub updated_ms: i64,
}

impl Validable for &Schema {
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
        let mut field_names = vec![];
        let mut duplicated_field_names = vec![];

        for schema_field in &self.fields {
            if field_names.contains(&&schema_field.label) {
                duplicated_field_names.push(schema_field.label.to_owned());
            }
            field_names.push(&schema_field.label);
        }

        if !duplicated_field_names.is_empty() {
            error_details.add_detail("fields", ("DUPLICATED_LABEL", duplicated_field_names));
        };

        Ok(())
    }
}

pub struct SchemaField {
    pub label: String,
    pub description: String,
    pub required: bool,
    pub field_validation: SchemaFieldValidation,
}

pub enum SchemaFieldValidation {
    Boolean {
        default: Option<bool>,
        value: SchemaFieldValue,
    },
    Number {
        min: Option<usize>,
        max: Option<usize>,
        default: Option<usize>,
        value: SchemaFieldValue,
    },
    Slug,
    String {
        min_length: Option<usize>,
        max_length: Option<usize>,
        default: Option<String>,
        value: SchemaFieldValue,
    },
}

pub enum SchemaFieldValue {
    Single { unique: bool },
    Localizable,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::content::{Content, ContentField, ContentFieldValue};
    use lightspeed_core::error::ErrorDetail;
    use lightspeed_core::service::validator::Validator;

    #[test]
    fn schema_validation_should_fail_if_fields_with_same_name() {
        let schema = Schema {
            name: "schema".to_owned(),
            updated_ms: 0,
            created_ms: 0,
            fields: vec![
                SchemaField {
                    label: "field1".to_owned(),
                    description: "".to_owned(),
                    field_validation: SchemaFieldValidation::Boolean {
                        value: SchemaFieldValue::Single { unique: false },
                        default: None,
                    },
                    required: false,
                },
                SchemaField {
                    label: "field2".to_owned(),
                    description: "".to_owned(),
                    field_validation: SchemaFieldValidation::Boolean {
                        value: SchemaFieldValue::Single { unique: false },
                        default: None,
                    },
                    required: false,
                },
                SchemaField {
                    label: "field2".to_owned(),
                    description: "".to_owned(),
                    field_validation: SchemaFieldValidation::Slug,
                    required: false,
                },
            ],
        };

        // Act
        let result = Validator::validate(&schema);

        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert_eq!(
                    details.details().get("fields"),
                    Some(&vec![ErrorDetail::new(
                        "DUPLICATED_LABEL",
                        vec!["field2".to_owned()]
                    )])
                );
            }
            _ => assert!(false),
        }
    }

}
