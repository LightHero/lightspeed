use crate::model::content::{ContentField, ContentFieldValue};
use lightspeed_core::error::ErrorDetails;
use lightspeed_core::service::validator::number::{validate_number_ge, validate_number_le};

pub struct Schema {
    pub name: String,
    pub fields: Vec<SchemaField>,
    pub created_ms: i64,
    pub updated_ms: i64,
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
    },
    Number {
        min: Option<usize>,
        max: Option<usize>,
        default: Option<usize>,
    },
    String {
        min_length: Option<usize>,
        max_length: Option<usize>,
        default: Option<String>,
    },
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
