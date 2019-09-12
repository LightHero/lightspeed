use lightspeed_core::error::ErrorDetails;
use crate::model::schema::{Schema, SchemaField, SchemaFieldValidation};
use lightspeed_core::service::validator::number::{validate_number_ge, validate_number_le};

pub struct Content {
    pub fields: Vec<ContentField>,
    pub created_ms: i64,
    pub updated_ms: i64,
}

pub struct ContentField {
    pub label: String,
    pub value: ContentFieldValue,
}

pub enum ContentFieldValue {
    Number(Option<usize>),
    String(Option<String>),
    Boolean(Option<bool>),
}

impl Content {
    pub fn validate(&self, schema: &Schema, error_details: &mut ErrorDetails) {
        let mut field_names = vec![];
        let mut duplicated_field_names = vec![];

        for content_field in &self.fields {
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

impl ContentField {
    pub fn validate(
        &self,
        schema_field: &SchemaField,
        error_details: &mut ErrorDetails,
    ) {
        let full_field_name = format!("fields.{}", self.label);
        match schema_field.field_validation {
            SchemaFieldValidation::Boolean { .. } => match &self.value {
                ContentFieldValue::Boolean(value) => {
                    if let None = value {
                        if schema_field.required {
                            error_details.add_detail(full_field_name, "VALUE_REQUIRED");
                        }
                    }
                }
                _ => error_details.add_detail(full_field_name, "SHOULD_BE_OF_TYPE_BOOLEAN"),
            },
            SchemaFieldValidation::Number { min, max, .. } => match &self.value {
                ContentFieldValue::Number(value) => {
                    if let Some(value) = value {
                        if let Some(min) = min {
                            validate_number_ge(
                                error_details,
                                &full_field_name,
                                min,
                                *value,
                            )
                        }
                        if let Some(max) = max {
                            validate_number_le(
                                error_details,
                                full_field_name,
                                max,
                                *value,
                            )
                        }
                    } else {
                        if schema_field.required {
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
            } => match &self.value {
                ContentFieldValue::String(value) => {
                    if let Some(value) = value {
                        if let Some(min_length) = min_length {
                            validate_number_ge(
                                error_details,
                                &full_field_name,
                                min_length,
                                value.len(),
                            )
                        }
                        if let Some(max_length) = max_length {
                            validate_number_le(
                                error_details,
                                full_field_name,
                                max_length,
                                value.len(),
                            )
                        }
                    } else {
                        if schema_field.required {
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
    use lightspeed_core::error::{ErrorDetail, LightSpeedError};
    use lightspeed_core::service::validator::Validator;
    use crate::model::schema::{SchemaField, SchemaFieldValidation, SchemaFieldValue};

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
            content.validate(&schema, error_details);
            Ok(())
        });

        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert_eq!(
                    details.details().get("fields"),
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
