use crate::data;
use c3p0::*;
use lightspeed_cms::dto::create_project_dto::CreateProjectDto;
use lightspeed_cms::dto::create_schema_dto::CreateSchemaDto;
use lightspeed_cms::model::project::ProjectData;
use lightspeed_cms::model::schema::Schema;
use lightspeed_cms::repository::CmsRepositoryManager;
use lightspeed_core::error::{ErrorDetail, LightSpeedError};
use lightspeed_core::service::validator::ERR_NOT_UNIQUE;
use lightspeed_core::utils::new_hyphenated_uuid;

#[test]
fn should_create_project() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let cms_module = &data.0;

    let c3p0 = cms_module.repo_manager.c3p0();
    let project_repo = cms_module.repo_manager.project_repo();

    let project = CreateProjectDto {
        name: new_hyphenated_uuid(),
    };

    let saved_project = cms_module.project_service.create_project(project)?;

    c3p0.transaction(|conn| {
    assert!(project_repo.exists_by_id(conn, &saved_project.id)?);
    assert!(cms_module
        .project_service
        .delete(saved_project.clone())
        .is_ok());
    assert!(!project_repo.exists_by_id(conn, &saved_project.id)?);

    Ok(())

    })
}

#[test]
fn project_name_should_be_unique() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let cms_module = &data.0;

    let c3p0 = cms_module.repo_manager.c3p0();
    let project_repo = cms_module.repo_manager.project_repo();

    let project = NewModel {
        version: 0,
        data: ProjectData {
            name: new_hyphenated_uuid(),
        },
    };

    c3p0.transaction(|conn| {
    assert!(project_repo
        .save(conn, project.clone())
        .is_ok());
    assert!(project_repo.save(conn, project).is_err());

    Ok(())

    })
}

#[test]
fn should_return_not_unique_validation_error() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let cms_module = &data.0;

    let project_service = &cms_module.project_service;

    let project = CreateProjectDto {
        name: new_hyphenated_uuid(),
    };

    assert!(project_service.create_project(project.clone()).is_ok());

    match project_service.create_project(project) {
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
}

#[test]
fn should_delete_all_schemas_when_project_is_deleted() -> Result<(), Box<dyn std::error::Error>> {
    let data = data(false);
    let cms_module = &data.0;

    // Arrange
    let c3p0 = cms_module.repo_manager.c3p0();
    let schema_repo = cms_module.repo_manager.schema_repo();
    let project_service = &cms_module.project_service;

    let project = CreateProjectDto {
        name: new_hyphenated_uuid(),
    };

    let saved_project = project_service.create_project(project)?;

    let mut schema = CreateSchemaDto {
        name: new_hyphenated_uuid(),
        project_id: saved_project.id,
        schema: Schema {
            created_ms: 0,
            updated_ms: 0,
            fields: vec![],
        },
    };

    let saved_schema_1 = cms_module.schema_service.create_schema(schema.clone())?;

    schema.name = new_hyphenated_uuid();
    let saved_schema_2 = cms_module.schema_service.create_schema(schema.clone())?;

    schema.project_id -= -1;
    let saved_schema_other = cms_module.schema_service.create_schema(schema)?;

    // Act
    assert!(project_service.delete(saved_project).is_ok());

    // Assert
    c3p0.transaction(|conn| {
        assert!(!schema_repo.exists_by_id(conn, &saved_schema_1.id)?);
        assert!(!schema_repo.exists_by_id(conn, &saved_schema_2.id)?);
        assert!(schema_repo.exists_by_id(conn, &saved_schema_other.id)?);
    
        Ok(())

    })
}
