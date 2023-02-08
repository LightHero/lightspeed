use crate::{data, test};
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
fn should_create_project() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let cms_module = &data.0;

        let c3p0 = cms_module.repo_manager.c3p0();
        let project_repo = cms_module.repo_manager.project_repo();

        let project = CreateProjectDto { name: new_hyphenated_uuid() };

        let saved_project = cms_module.project_service.create_project(project).await?;

        c3p0.transaction(|mut conn| async move {
            let conn = &mut conn;
            assert!(project_repo.exists_by_id(conn, &saved_project.id).await?);
            assert!(cms_module.project_service.delete(saved_project.clone()).await.is_ok());
            assert!(!project_repo.exists_by_id(conn, &saved_project.id).await?);

            Ok(())
        })
        .await
    })
}

#[test]
fn project_name_should_be_unique() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let cms_module = &data.0;

        let c3p0 = cms_module.repo_manager.c3p0();
        let project_repo = cms_module.repo_manager.project_repo();

        let project = NewModel { version: 0, data: ProjectData { name: new_hyphenated_uuid() } };

        c3p0.transaction(|mut conn| async move {
            let conn = &mut conn;
            assert!(project_repo.save(conn, project.clone()).await.is_ok());
            assert!(project_repo.save(conn, project).await.is_err());

            Ok(())
        })
        .await
    })
}

#[test]
fn should_return_not_unique_validation_error() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let cms_module = &data.0;

        let project_service = &cms_module.project_service;

        let project = CreateProjectDto { name: new_hyphenated_uuid() };

        assert!(project_service.create_project(project.clone()).await.is_ok());

        match project_service.create_project(project).await {
            Err(LightSpeedError::ValidationError { details }) => {
                assert_eq!(details.details.len(), 1);
                assert_eq!(details.details.get("name").unwrap()[0], ErrorDetail::from(ERR_NOT_UNIQUE));
            }
            _ => panic!(),
        };

        Ok(())
    })
}

#[test]
fn should_delete_all_schemas_when_project_is_deleted() -> Result<(), LightSpeedError> {
    test(async {
        let data = data(false).await;
        let cms_module = &data.0;

        // Arrange
        let c3p0 = cms_module.repo_manager.c3p0();
        let schema_repo = cms_module.repo_manager.schema_repo();
        let project_service = &cms_module.project_service;

        let project = CreateProjectDto { name: new_hyphenated_uuid() };

        let saved_project = project_service.create_project(project).await?;

        let mut schema = CreateSchemaDto {
            name: new_hyphenated_uuid(),
            project_id: saved_project.id,
            schema: Schema { created_ms: 0, updated_ms: 0, fields: vec![] },
        };

        let saved_schema_1 = cms_module.schema_service.create_schema(schema.clone()).await?;

        schema.name = new_hyphenated_uuid();
        let saved_schema_2 = cms_module.schema_service.create_schema(schema.clone()).await?;

        schema.project_id -= -1;
        let saved_schema_other = cms_module.schema_service.create_schema(schema).await?;

        // Act
        assert!(project_service.delete(saved_project).await.is_ok());

        // Assert
        c3p0.transaction(|mut conn| async move {
            let conn = &mut conn;
            assert!(!schema_repo.exists_by_id(conn, &saved_schema_1.id).await?);
            assert!(!schema_repo.exists_by_id(conn, &saved_schema_2.id).await?);
            assert!(schema_repo.exists_by_id(conn, &saved_schema_other.id).await?);

            Ok(())
        })
        .await
    })
}
