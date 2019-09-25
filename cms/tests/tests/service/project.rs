use crate::test;
use c3p0::*;
use lightspeed_cms::repository::CmsRepositoryManager;
use lightspeed_cms::model::project::ProjectData;
use lightspeed_core::utils::new_hyphenated_uuid;
use lightspeed_core::error::{LightSpeedError, ErrorDetail};
use lightspeed_core::service::validator::ERR_NOT_UNIQUE;
use lightspeed_cms::dto::create_project_dto::CreateProjectDto;

#[test]
fn should_create_project() {
    test(|cms_module| {
        let c3p0 = cms_module.repo_manager.c3p0();
        let project_repo = cms_module.repo_manager.project_repo();

        let project = CreateProjectDto {
            name: new_hyphenated_uuid(),
        };

        let saved_project = cms_module.project_service.create_project(project)?;

        assert!(project_repo.exists_by_id(&c3p0.connection()?, &saved_project.id)?);
        assert_eq!(
            1,
            cms_module
                .project_service
                .delete(saved_project.clone())?
        );
        assert!(!project_repo.exists_by_id(&c3p0.connection()?, &saved_project.id)?);

        Ok(())
    });
}

#[test]
fn project_name_should_be_unique() {
    test(|cms_module| {
        let c3p0 = cms_module.repo_manager.c3p0();
        let project_repo = cms_module.repo_manager.project_repo();

        let project = NewModel {
            version: 0,
            data: ProjectData {
                name: new_hyphenated_uuid(),
            },
        };

        assert!(project_repo.save(&c3p0.connection()?, project.clone()).is_ok());
        assert!(project_repo.save(&c3p0.connection()?, project).is_err());

        Ok(())
    });
}

#[test]
fn should_return_not_unique_validation_error() {
    test(|cms_module| {
        let project_service = &cms_module.project_service;

        let project = CreateProjectDto {
            name: new_hyphenated_uuid(),
        };

        assert!(project_service.create_project(project.clone()).is_ok());

        match project_service.create_project(project) {
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