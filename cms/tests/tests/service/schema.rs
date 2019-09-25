use crate::test;
use c3p0::*;
use lightspeed_cms::repository::CmsRepositoryManager;
use lightspeed_core::utils::new_hyphenated_uuid;
use lightspeed_core::error::{LightSpeedError, ErrorDetail};
use lightspeed_core::service::validator::ERR_NOT_UNIQUE;
use lightspeed_cms::dto::create_schema_dto::CreateSchemaDto;
use lightspeed_cms::model::schema::Schema;

#[test]
fn should_create_schema() {
    test(|cms_module| {
        let c3p0 = cms_module.repo_manager.c3p0();
        let schema_repo = cms_module.repo_manager.schema_repo();

        let schema = CreateSchemaDto {
            name: new_hyphenated_uuid(),
            project_id: 0,
            schema: Schema {
                created_ms: 0,
                updated_ms: 0,
                fields: vec![]
            }
        };

        let saved_schema = cms_module.schema_service.create_schema(schema)?;

        assert!(schema_repo.exists_by_id(&c3p0.connection()?, &saved_schema.id)?);
        assert_eq!(
            1,
            cms_module
                .schema_service
                .delete(saved_schema.clone())?
        );
        assert!(!schema_repo.exists_by_id(&c3p0.connection()?, &saved_schema.id)?);

        Ok(())
    });
}

#[test]
fn schema_name_should_be_unique_per_project() {
    test(|cms_module| {
        let c3p0 = cms_module.repo_manager.c3p0();
        let schema_repo = cms_module.repo_manager.schema_repo();

        let mut schema = CreateSchemaDto {
            name: new_hyphenated_uuid(),
            project_id: 0,
            schema: Schema {
                created_ms: 0,
                updated_ms: 0,
                fields: vec![]
            }
        };

        assert!(schema_repo.save(&c3p0.connection()?, NewModel::new(schema.clone())).is_ok());
        assert!(schema_repo.save(&c3p0.connection()?, NewModel::new(schema.clone())).is_err());

        schema.project_id = 1;

        assert!(schema_repo.save(&c3p0.connection()?, NewModel::new(schema.clone())).is_ok());

        Ok(())
    });
}

#[test]
fn should_return_not_unique_validation_error() {
    test(|cms_module| {
        let schema_service = &cms_module.schema_service;

        let schema = CreateSchemaDto {
            name: new_hyphenated_uuid(),
            project_id: 0,
            schema: Schema {
                created_ms: 0,
                updated_ms: 0,
                fields: vec![]
            }
        };

        assert!(schema_service.create_schema(schema.clone()).is_ok());

        match schema_service.create_schema(schema) {
            Err(LightSpeedError::ValidationError {details}) => {
                assert_eq!(details.details().borrow().len(), 1);
                assert_eq!(
                    details.details().borrow().get("name").unwrap()[0],
                    ErrorDetail::from(ERR_NOT_UNIQUE)
                );
            },
            _ => assert!(false)
        };

        Ok(())
    });
}