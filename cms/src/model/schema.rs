use c3p0::Model;
use lightspeed_core::error::{ErrorDetails, LightSpeedError};
use lightspeed_core::service::validator::order::{validate_ge, validate_le};
use lightspeed_core::service::validator::{Validable, ERR_NOT_UNIQUE};
use once_cell::sync::OnceCell;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub type SchemaModel = Model<SchemaData>;
pub const SCHEMA_FIELD_NAME_MAX_LENGHT: usize = 32;
pub const SCHAME_FIELD_NAME_VALIDATION_REGEX: &str = r#"^[a-z0-9_]+$"#;

const NOT_VALID_FIELD_NAME: &str = "NOT_VALID_FIELD_NAME";

pub fn field_name_regex() -> &'static Regex {
    static REGEX: OnceCell<Regex> = OnceCell::new();
    REGEX.get_or_init(|| {
        Regex::new(SCHAME_FIELD_NAME_VALIDATION_REGEX).expect("field name validation regex should be valid")
    })
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SchemaData {
    pub name: String,
    pub project_id: i64,
    pub schema: Schema,
}

impl Validable for SchemaData {
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
        validate_ge(error_details, "name", 3, self.name.len());
        (&self.schema).validate(&mut error_details.with_scope("schema"))?;
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Schema {
    pub fields: Vec<SchemaField>,
    pub created_ms: i64,
    pub updated_ms: i64,
}

impl Validable for Schema {
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
        let mut field_names = vec![];

        for (count, schema_field) in (&self.fields).iter().enumerate() {
            let mut scoped_err = error_details.with_scope(format!("fields[{}]", count));
            if field_names.contains(&&schema_field.name) {
                scoped_err.add_detail("name", ERR_NOT_UNIQUE);
            }
            field_names.push(&schema_field.name);
            schema_field.validate(&mut scoped_err)?;
        }

        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SchemaField {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub field_type: SchemaFieldType,
}

impl Validable for SchemaField {
    fn validate(&self, error_details: &mut ErrorDetails) -> Result<(), LightSpeedError> {
        validate_ge(error_details, "name", 1, self.name.len());
        validate_le(error_details, "name", SCHEMA_FIELD_NAME_MAX_LENGHT, self.name.len());

        if !field_name_regex().is_match(&self.name) {
            error_details.add_detail("name", NOT_VALID_FIELD_NAME);
        }

        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum SchemaFieldType {
    Boolean { default: Option<bool>, arity: SchemaFieldArity },
    Number { min: Option<u64>, max: Option<u64>, default: Option<u64>, arity: SchemaFieldArity },
    Slug,
    String { min_length: Option<usize>, max_length: Option<usize>, default: Option<String>, arity: SchemaFieldArity },
}

impl SchemaFieldType {
    pub fn get_arity(&self) -> &SchemaFieldArity {
        match self {
            SchemaFieldType::Boolean { arity, .. }
            | SchemaFieldType::Number { arity, .. }
            | SchemaFieldType::String { arity, .. } => arity,
            SchemaFieldType::Slug => &SchemaFieldArity::Unique,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum SchemaFieldArity {
    Unique,
    Single,
    Localizable { options: LocalizableOptions },
}

#[derive(Clone, Serialize, Deserialize)]
pub enum LocalizableOptions {
    Languages { languages: Vec<String> },
}

#[cfg(test)]
mod test {
    use super::*;
    use lightspeed_core::error::ErrorDetail;
    use lightspeed_core::service::validator::order::MUST_BE_GREATER_OR_EQUAL;
    use lightspeed_core::service::validator::Validator;

    #[test]
    fn schema_validation_should_fail_if_name_is_empty() {
        let schema_data = SchemaData {
            name: "1".to_owned(),
            project_id: 0,
            schema: Schema {
                updated_ms: 0,
                created_ms: 0,
                fields: vec![
                    SchemaField {
                        name: "label1".to_owned(),
                        description: "".to_owned(),
                        field_type: SchemaFieldType::Boolean { arity: SchemaFieldArity::Single, default: None },
                        required: false,
                    },
                    SchemaField {
                        name: "label2".to_owned(),
                        description: "".to_owned(),
                        field_type: SchemaFieldType::Boolean { arity: SchemaFieldArity::Single, default: None },
                        required: false,
                    },
                    SchemaField {
                        name: "label2".to_owned(),
                        description: "".to_owned(),
                        field_type: SchemaFieldType::Slug,
                        required: false,
                    },
                ],
            },
        };

        // Act
        let result = Validator::validate(&schema_data);

        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert_eq!(details.details.len(), 2);
                assert_eq!(
                    details.details.get("name"),
                    Some(&vec![ErrorDetail::new(MUST_BE_GREATER_OR_EQUAL, vec!["3".to_owned()])])
                );
                assert_eq!(
                    details.details.get("schema.fields[2].name"),
                    Some(&vec![ErrorDetail::new(ERR_NOT_UNIQUE, vec![])])
                );
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn schema_validation_should_fail_if_fields_with_same_name() {
        let schema = Schema {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![
                SchemaField {
                    name: "label1".to_owned(),
                    description: "".to_owned(),
                    field_type: SchemaFieldType::Boolean { arity: SchemaFieldArity::Single, default: None },
                    required: false,
                },
                SchemaField {
                    name: "".to_owned(),
                    description: "".to_owned(),
                    field_type: SchemaFieldType::Boolean { arity: SchemaFieldArity::Unique, default: None },
                    required: false,
                },
            ],
        };

        // Act
        let result = Validator::validate(&schema);

        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert_eq!(details.details.len(), 1);
                assert_eq!(
                    details.details.get("fields[1].name"),
                    Some(&vec![
                        ErrorDetail::new(MUST_BE_GREATER_OR_EQUAL, vec!["1".to_owned()]),
                        ErrorDetail::new(NOT_VALID_FIELD_NAME, vec![]),
                    ])
                );
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn schema_field_names_should_contain_only_lowercased_chars_and_undercores() {
        let schema = Schema {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![
                SchemaField {
                    name: "label1".to_owned(),
                    description: "".to_owned(),
                    field_type: SchemaFieldType::Slug,
                    required: false,
                },
                SchemaField {
                    name: "SomeUppercase".to_owned(),
                    description: "".to_owned(),
                    field_type: SchemaFieldType::Slug,
                    required: false,
                },
                SchemaField {
                    name: "1123".to_owned(),
                    description: "".to_owned(),
                    field_type: SchemaFieldType::Slug,
                    required: false,
                },
                SchemaField {
                    name: "with whitespaces".to_owned(),
                    description: "".to_owned(),
                    field_type: SchemaFieldType::Slug,
                    required: false,
                },
            ],
        };

        // Act
        let result = Validator::validate(&schema);

        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert_eq!(details.details.len(), 2);
                assert_eq!(
                    details.details.get("fields[1].name"),
                    Some(&vec![ErrorDetail::new(NOT_VALID_FIELD_NAME, vec![]),])
                );
                assert_eq!(
                    details.details.get("fields[3].name"),
                    Some(&vec![ErrorDetail::new(NOT_VALID_FIELD_NAME, vec![]),])
                );
            }
            _ => assert!(false),
        }
    }
}
