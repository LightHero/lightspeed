use c3p0::Model;
use lightspeed_core::error::{ErrorDetails, LightSpeedError};
use lightspeed_core::service::validator::number::validate_number_ge;
use lightspeed_core::service::validator::Validable;
use serde::{Deserialize, Serialize};

const MUST_BE_UNIQUE: &str = "MUST_BE_UNIQUE";

pub type SchemaModel = Model<SchemaData>;

#[derive(Clone, Serialize, Deserialize)]
pub struct SchemaData {
    pub name: String,
    pub project_name: String,
    pub schema: Schema,
}

impl Validable for &SchemaData {
    fn validate(&self, error_details: &ErrorDetails) -> Result<(), LightSpeedError> {
        validate_number_ge(error_details, "name", 3, self.name.len());
        (&self.schema).validate(&error_details.with_scope("schema"))?;
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Schema {
    pub fields: Vec<SchemaField>,
    pub created_ms: i64,
    pub updated_ms: i64,
}

impl Validable for &Schema {
    fn validate(&self, error_details: &ErrorDetails) -> Result<(), LightSpeedError> {
        let mut field_names = vec![];

        for (count, schema_field) in (&self.fields).iter().enumerate() {
            let scoped_err = error_details.with_scope(format!("fields[{}]", count));
            if field_names.contains(&&schema_field.name) {
                scoped_err.add_detail("name", MUST_BE_UNIQUE);
            }
            field_names.push(&schema_field.name);
            schema_field.validate(&scoped_err)?;
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

impl Validable for &SchemaField {
    fn validate(&self, error_details: &ErrorDetails) -> Result<(), LightSpeedError> {
        validate_number_ge(error_details, "name", 1, self.name.len());
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum SchemaFieldType {
    Boolean {
        default: Option<bool>,
        arity: SchemaFieldArity,
    },
    Number {
        min: Option<usize>,
        max: Option<usize>,
        default: Option<usize>,
        arity: SchemaFieldArity,
    },
    Slug,
    String {
        min_length: Option<usize>,
        max_length: Option<usize>,
        default: Option<String>,
        arity: SchemaFieldArity,
    },
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
    use lightspeed_core::service::validator::number::MUST_BE_GREATER_OR_EQUAL;
    use lightspeed_core::service::validator::Validator;

    #[test]
    fn schema_validation_should_fail_if_name_is_empty() {
        let schema_data = SchemaData {
            name: "1".to_owned(),
            project_name: "".to_owned(),
            schema: Schema {
                updated_ms: 0,
                created_ms: 0,
                fields: vec![
                    SchemaField {
                        name: "label1".to_owned(),
                        description: "".to_owned(),
                        field_type: SchemaFieldType::Boolean {
                            arity: SchemaFieldArity::Single,
                            default: None,
                        },
                        required: false,
                    },
                    SchemaField {
                        name: "label2".to_owned(),
                        description: "".to_owned(),
                        field_type: SchemaFieldType::Boolean {
                            arity: SchemaFieldArity::Single,
                            default: None,
                        },
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
                assert_eq!(details.details().borrow().len(), 2);
                assert_eq!(
                    details.details().borrow().get("name"),
                    Some(&vec![ErrorDetail::new(
                        MUST_BE_GREATER_OR_EQUAL,
                        vec!["3".to_owned()]
                    )])
                );
                assert_eq!(
                    details.details().borrow().get("schema.fields[2].name"),
                    Some(&vec![ErrorDetail::new(MUST_BE_UNIQUE, vec![])])
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
                    field_type: SchemaFieldType::Boolean {
                        arity: SchemaFieldArity::Single,
                        default: None,
                    },
                    required: false,
                },
                SchemaField {
                    name: "".to_owned(),
                    description: "".to_owned(),
                    field_type: SchemaFieldType::Boolean {
                        arity: SchemaFieldArity::Unique,
                        default: None,
                    },
                    required: false,
                },
            ],
        };

        // Act
        let result = Validator::validate(&schema);

        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert_eq!(details.details().borrow().len(), 1);
                assert_eq!(
                    details.details().borrow().get("fields[1].name"),
                    Some(&vec![ErrorDetail::new(
                        MUST_BE_GREATER_OR_EQUAL,
                        vec!["1".to_owned()]
                    )])
                );
            }
            _ => assert!(false),
        }
    }

}
