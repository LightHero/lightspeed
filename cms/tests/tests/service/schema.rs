use crate::data;
use c3p0::*;
use lightspeed_cms::dto::create_schema_dto::CreateSchemaDto;
use lightspeed_cms::model::schema::Schema;
use lightspeed_cms::repository::CmsRepositoryManager;
use lightspeed_core::error::{ErrorDetail, LightSpeedError};
use lightspeed_core::service::validator::ERR_NOT_UNIQUE;
use lightspeed_core::utils::new_hyphenated_uuid;

#[test]
fn should_create_schema() -> Result<(), LightSpeedError> {
        test(async {
        let data = data(false).await;
    let cms_module = &data.0;

    let c3p0 = cms_module.repo_manager.c3p0();
    let schema_repo = cms_module.repo_manager.schema_repo();

    let schema = CreateSchemaDto {
        name: new_hyphenated_uuid(),
        project_id: -1,
        schema: Schema {
            created_ms: 0,
            updated_ms: 0,
            fields: vec![],
        },
    };

    let saved_schema = cms_module.schema_service.create_schema(schema)?;

    c3p0.transaction(|mut conn| async {
        let conn = &mut conn;
    assert!(schema_repo.exists_by_id(conn, &saved_schema.id)?);
    assert!(cms_module
        .schema_service
        .delete(saved_schema.clone())
        .is_ok());
    assert!(!schema_repo.exists_by_id(conn, &saved_schema.id)?);

    Ok(())

    })
        })
}

#[test]
fn schema_name_should_be_unique_per_project() -> Result<(), LightSpeedError> {
        test(async {
        let data = data(false).await;
    let cms_module = &data.0;

    let c3p0 = cms_module.repo_manager.c3p0();
    let schema_repo = cms_module.repo_manager.schema_repo();

    let mut schema = CreateSchemaDto {
        name: new_hyphenated_uuid(),
        project_id: -1,
        schema: Schema {
            created_ms: 0,
            updated_ms: 0,
            fields: vec![],
        },
    };

    c3p0.transaction::<_,C3p0Error,_>(|mut conn| {
        assert!(schema_repo
            .save(conn, NewModel::new(schema.clone()))
            .is_ok());
        assert!(schema_repo
            .save(conn, NewModel::new(schema.clone()))
            .is_err());
        Ok(())
    })?;
        schema.project_id = -2;

        c3p0.transaction(|mut conn| async {
        assert!(schema_repo
            .save(conn, NewModel::new(schema.clone()))
            .is_ok());

        Ok(())

    })
        })
}

#[test]
fn should_return_not_unique_validation_error() -> Result<(), LightSpeedError> {
        test(async {
        let data = data(false).await;
    let cms_module = &data.0;

    let schema_service = &cms_module.schema_service;

    let schema = CreateSchemaDto {
        name: new_hyphenated_uuid(),
        project_id: -1,
        schema: Schema {
            created_ms: 0,
            updated_ms: 0,
            fields: vec![],
        },
    };

    assert!(schema_service.create_schema(schema.clone()).is_ok());

    match schema_service.create_schema(schema) {
        Err(LightSpeedError::ValidationError { details }) => {
            assert_eq!(details.details.len(), 1);
            assert_eq!(
                details.details.get("name").unwrap()[0],
                ErrorDetail::from(ERR_NOT_UNIQUE)
            );
        }
        _ => assert!(false),
    };

    Ok(())
        })
}

#[test]
fn should_delete_schemas_by_project_id() -> Result<(), LightSpeedError> {
        test(async {
        let data = data(false).await;
    let cms_module = &data.0;

    // Arrange
    let c3p0 = cms_module.repo_manager.c3p0();
    let schema_repo = cms_module.repo_manager.schema_repo();

    let project_id = -10;

    let mut schema = CreateSchemaDto {
        name: new_hyphenated_uuid(),
        project_id,
        schema: Schema {
            created_ms: 0,
            updated_ms: 0,
            fields: vec![],
        },
    };

    let saved_schema_1 = cms_module.schema_service.create_schema(schema.clone())?;

    schema.name = new_hyphenated_uuid();
    let saved_schema_2 = cms_module.schema_service.create_schema(schema.clone())?;

    schema.project_id = project_id - 1;
    let saved_schema_other = cms_module.schema_service.create_schema(schema)?;

    // Act
    c3p0.transaction(|mut conn| async {
        let conn = &mut conn;
        assert_eq!(
            2,
            cms_module
                .schema_service
                .delete_by_project_id(conn, project_id)?
        );

        // Assert
        assert!(!schema_repo.exists_by_id(conn, &saved_schema_1.id)?);
        assert!(!schema_repo.exists_by_id(conn, &saved_schema_2.id)?);
        assert!(schema_repo.exists_by_id(conn, &saved_schema_other.id)?);

        Ok(())
    })
        })
}
