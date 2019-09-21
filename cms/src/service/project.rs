use crate::repository::ProjectRepository;

#[derive(Clone)]
pub struct ProjectService<ProjectRepo: ProjectRepository> {
    project_repo: ProjectRepo,
}

impl<ProjectRepo: ProjectRepository> ProjectService<ProjectRepo> {
    pub fn new(project_repo: ProjectRepo) -> Self {
        ProjectService {
            project_repo,
        }
    }
}

#[cfg(test)]
pub mod test {

}
