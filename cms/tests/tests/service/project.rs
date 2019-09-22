use crate::test;
use c3p0::*;
use lightspeed_cms::repository::CmsRepositoryManager;
use lightspeed_cms::model::project::ProjectData;

#[test]
fn should_delete_token() {
    test(|cms_module| {
        let c3p0 = cms_module.repo_manager.c3p0();
        let project_repo = cms_module.repo_manager.project_repo();

        let procject = NewModel {
            version: 0,
            data: ProjectData {
                name: "project_name".to_owned(),
            },
        };

        let saved_token = project_repo.save(&c3p0.connection()?, token)?;

        assert!(token_repo.exists_by_id(&c3p0.connection()?, &saved_token.id)?);
        assert_eq!(
            1,
            cms_module
                .token_service
                .delete(&c3p0.connection()?, saved_token.clone())?
        );
        assert!(!token_repo.exists_by_id(&c3p0.connection()?, &saved_token.id)?);

        Ok(())
    });
}
