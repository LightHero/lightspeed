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

impl Schema {
    pub fn validate_content(&self, content: &Content, error_details: &mut ErrorDetails) {
        let mut field_names = vec![];
        let mut duplicated_field_names = vec![];

        for content_field in &content.fields {
            if field_names.contains(&&content_field.label) {
                duplicated_field_names.push(content_field.label.to_owned());
            }
            field_names.push(&content_field.label);
        }

        if !duplicated_field_names.is_empty() {
            error_details.add_detail("fields", ("DUPLICATED_LABEL", duplicated_field_names));
        }
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

impl SchemaFieldValidation {
    pub fn validate_content_field(
        &self,
        content_field: &ContentField,
        required: bool,
        error_details: &mut ErrorDetails,
    ) {
        let full_field_name = format!("fields.{}", content_field.label);
        match self {
            SchemaFieldValidation::Boolean { .. } => match &content_field.value {
                ContentFieldValue::Boolean(value) => {
                    if let None = value {
                        if required {
                            error_details.add_detail(full_field_name, "VALUE_REQUIRED");
                        }
                    }
                }
                _ => error_details.add_detail(full_field_name, "SHOULD_BE_OF_TYPE_BOOLEAN"),
            },
            SchemaFieldValidation::Number { min, max, .. } => match &content_field.value {
                ContentFieldValue::Number(value) => {
                    if let Some(value) = value {
                        if let Some(min) = min {
                            validate_number_ge(
                                error_details,
                                format!("fields.{}", content_field.label),
                                *min,
                                *value,
                            )
                        }
                        if let Some(max) = max {
                            validate_number_le(
                                error_details,
                                format!("fields.{}", content_field.label),
                                *max,
                                *value,
                            )
                        }
                    } else {
                        if required {
                            error_details.add_detail(full_field_name, "VALUE_REQUIRED");
                        }
                    }
                }
                _ => error_details.add_detail(full_field_name, "SHOULD_BE_OF_TYPE_NUMBER"),
            },
            SchemaFieldValidation::Slug => unimplemented!(),
            SchemaFieldValidation::String {
                min_length,
                max_length,
                ..
            } => match &content_field.value {
                ContentFieldValue::String(value) => {
                    if let Some(value) = value {
                        if let Some(min_length) = min_length {
                            validate_number_ge(
                                error_details,
                                format!("fields.{}", content_field.label),
                                *min_length,
                                value.len(),
                            )
                        }
                        if let Some(max_length) = max_length {
                            validate_number_le(
                                error_details,
                                format!("fields.{}", content_field.label),
                                *max_length,
                                value.len(),
                            )
                        }
                    } else {
                        if required {
                            error_details.add_detail(full_field_name, "VALUE_REQUIRED");
                        }
                    }
                }
                _ => error_details.add_detail(full_field_name, "SHOULD_BE_OF_TYPE_STRING"),
            },
        }
    }
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
                    details.details.get("fields"),
                    Some(&vec![ErrorDetail::new(
                        "DUPLICATED_LABEL",
                        vec!["field2".to_owned()]
                    )])
                );
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn content_validation_should_fail_if_fields_with_same_name() {
        // Arrange
        let mut content = Content {
            created_ms: 0,
            updated_ms: 0,
            fields: vec![],
        };
        content.fields.push(ContentField {
            label: "one".to_owned(),
            value: ContentFieldValue::Boolean(None),
        });
        content.fields.push(ContentField {
            label: "one".to_owned(),
            value: ContentFieldValue::Boolean(None),
        });
        content.fields.push(ContentField {
            label: "two".to_owned(),
            value: ContentFieldValue::Boolean(None),
        });

        let schema = Schema {
            name: "schema".to_owned(),
            updated_ms: 0,
            created_ms: 0,
            fields: vec![SchemaField {
                label: "field".to_owned(),
                description: "".to_owned(),
                field_validation: SchemaFieldValidation::Boolean {
                    value: SchemaFieldValue::Single { unique: false },
                    default: None,
                },
                required: false,
            }],
        };

        // Act
        let result = Validator::validate(|error_details: &mut ErrorDetails| {
            schema.validate_content(&content, error_details);
            Ok(())
        });

        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert_eq!(
                    details.details.get("fields"),
                    Some(&vec![ErrorDetail::new(
                        "DUPLICATED_LABEL",
                        vec!["one".to_owned()]
                    )])
                );
            }
            _ => assert!(false),
        }
    }

}
