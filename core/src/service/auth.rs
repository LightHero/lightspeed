use crate::error::LightSpeedError;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use typescript_definitions::TypeScriptify;

#[derive(Debug, Clone, Serialize, Deserialize, TypeScriptify)]
#[serde(rename_all = "camelCase")]
pub struct Auth {
    pub id: i64,
    pub username: String,
    pub roles: Vec<String>,
}

impl Auth {
    pub fn new<S: Into<String>>(id: i64, username: S, roles: Vec<String>) -> Self {
        Self { id, username: username.into(), roles }
    }
}

impl Default for Auth {
    fn default() -> Self {
        Self { id: -1, username: "".to_owned(), roles: vec![] }
    }
}

#[derive(Clone)]
pub struct Role {
    pub name: String,
    pub permissions: Vec<String>,
}

pub trait Owned {
    fn get_owner_id(&self) -> i64;
}

#[derive(Clone)]
pub struct AuthService<T: RolesProvider> {
    roles_provider: T,
}

impl<T: RolesProvider> AuthService<T> {
    pub fn new(roles_provider: T) -> AuthService<T> {
        AuthService { roles_provider }
    }

    pub fn auth(&self, auth: Auth) -> AuthContext {
        AuthContext::new(auth, &self.roles_provider)
    }
}

pub struct AuthContext<'a> {
    pub auth: Auth,
    permissions: Vec<&'a str>,
}

impl<'a> AuthContext<'a> {
    pub fn new<T: RolesProvider>(auth: Auth, roles_provider: &T) -> AuthContext {
        let permissions = roles_provider.get_permissions_by_role_name(&auth.roles);
        AuthContext { auth, permissions }
    }

    pub fn is_authenticated(&self) -> Result<&AuthContext, LightSpeedError> {
        if self.auth.username.is_empty() {
            return Err(LightSpeedError::UnauthenticatedError {});
        };
        Ok(&self)
    }

    pub fn has_role(&self, role: &str) -> Result<&AuthContext, LightSpeedError> {
        self.is_authenticated()?;
        if !self.has_role_bool(&role) {
            return Err(LightSpeedError::ForbiddenError {
                message: format!("User [{}] does not have the required role [{}]", self.auth.id, role),
            });
        };
        Ok(&self)
    }

    pub fn has_any_role(&self, roles: &[&str]) -> Result<&AuthContext, LightSpeedError> {
        self.is_authenticated()?;
        for role in roles {
            if self.has_role_bool(*role) {
                return Ok(&self);
            };
        }
        Err(LightSpeedError::ForbiddenError { message: format!("User [{}] does not have the required role", self.auth.id) })
    }

    pub fn has_all_roles(&self, roles: &[&str]) -> Result<&AuthContext, LightSpeedError> {
        self.is_authenticated()?;
        for role in roles {
            if !self.has_role_bool(*role) {
                return Err(LightSpeedError::ForbiddenError {
                    message: format!("User [{}] does not have the required role [{}]", self.auth.id, role),
                });
            };
        }
        Ok(&self)
    }

    pub fn has_permission(&self, permission: &str) -> Result<&AuthContext, LightSpeedError> {
        self.is_authenticated()?.has_permission_bool(&permission);

        if !self.has_permission_bool(&permission) {
            return Err(LightSpeedError::ForbiddenError {
                message: format!("User [{}] does not have the required permission [{}]", self.auth.id, permission),
            });
        };
        Ok(&self)
    }

    pub fn has_any_permission(&self, permissions: &[&str]) -> Result<&AuthContext, LightSpeedError> {
        self.is_authenticated()?;
        for permission in permissions {
            if self.has_permission_bool(*permission) {
                return Ok(&self);
            };
        }
        Err(LightSpeedError::ForbiddenError { message: format!("User [{}] does not have the required permission", self.auth.id) })
    }

    pub fn has_all_permissions(&self, permissions: &[&str]) -> Result<&AuthContext, LightSpeedError> {
        self.is_authenticated()?;
        for permission in permissions {
            if !self.has_permission_bool(*permission) {
                return Err(LightSpeedError::ForbiddenError {
                    message: format!("User [{}] does not have the required permission [{}]", self.auth.id, permission),
                });
            };
        }
        Ok(&self)
    }

    pub fn is_owner<T: Owned>(&self, obj: &T) -> Result<&AuthContext, LightSpeedError> {
        if self.auth.id == obj.get_owner_id() {
            Ok(&self)
        } else {
            Err(LightSpeedError::ForbiddenError {
                message: format!(
                    "User [{}] is not the owner. User id [{}], owner id: [{}]",
                    self.auth.id,
                    self.auth.id,
                    obj.get_owner_id()
                ),
            })
        }
    }

    pub fn is_owner_or_has_role<T: Owned>(&self, obj: &T, role: &str) -> Result<&AuthContext, LightSpeedError> {
        if (self.auth.id == obj.get_owner_id()) || self.has_role_bool(role) {
            Ok(&self)
        } else {
            Err(LightSpeedError::ForbiddenError {
                message: format!(
                    "User [{}] is not the owner and does not have role [{}]. User id [{}], owner id: [{}]",
                    self.auth.id,
                    role,
                    self.auth.id,
                    obj.get_owner_id()
                ),
            })
        }
    }

    pub fn is_owner_or_has_permission<T: Owned>(&self, obj: &T, permission: &str) -> Result<&AuthContext, LightSpeedError> {
        if (self.auth.id == obj.get_owner_id()) || self.has_permission_bool(permission) {
            Ok(&self)
        } else {
            Err(LightSpeedError::ForbiddenError {
                message: format!(
                    "User [{}] is not the owner and does not have permission [{}]. User id [{}], owner id: [{}]",
                    self.auth.id,
                    permission,
                    self.auth.id,
                    obj.get_owner_id()
                ),
            })
        }
    }

    fn has_role_bool(&self, role: &str) -> bool {
        self.auth.roles.iter().any(|x| x == role)
    }

    fn has_permission_bool(&self, permission: &str) -> bool {
        self.permissions.contains(&permission)
    }
}

pub trait RolesProvider: Send + Sync + Clone {
    fn get_all(&self) -> &[Role];

    fn get_by_name(&self, names: &[String]) -> Vec<&Role>;

    fn get_permissions_by_role_name(&self, role_names: &[String]) -> Vec<&str>;
}

#[derive(Clone)]
pub struct InMemoryRolesProvider {
    all_roles: Arc<[Role]>,
    roles_by_name: Arc<HashMap<String, Role>>,
}

impl InMemoryRolesProvider {
    pub fn new(all_roles: Vec<Role>) -> InMemoryRolesProvider {
        let mut roles_by_name = HashMap::new();

        for role in all_roles.iter() {
            roles_by_name.insert(role.name.clone(), role.clone());
        }

        InMemoryRolesProvider { all_roles: all_roles.into(), roles_by_name: Arc::new(roles_by_name) }
    }
}

impl RolesProvider for InMemoryRolesProvider {
    fn get_all(&self) -> &[Role] {
        &self.all_roles
    }

    fn get_by_name(&self, names: &[String]) -> Vec<&Role> {
        let mut result = vec![];
        for name in names {
            let roles = self.roles_by_name.get(name);
            if let Some(t) = roles {
                result.push(t)
            }
        }
        result
    }

    fn get_permissions_by_role_name(&self, role_names: &[String]) -> Vec<&str> {
        let mut permissions = vec![];
        for name in role_names {
            if let Some(role) = self.roles_by_name.get(name) {
                for permission in &role.permissions {
                    permissions.push(permission.as_str())
                }
            }
        }
        permissions
    }
}

#[cfg(test)]
mod test_role_provider {
    use super::Role;
    use super::RolesProvider;

    #[test]
    fn should_return_all_roles() {
        let roles =
            vec![Role { name: "RoleOne".to_string(), permissions: vec![] }, Role { name: "RoleTwo".to_string(), permissions: vec![] }];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let get_all = provider.get_all();
        assert!(!get_all.is_empty());
        assert_eq!(roles.len(), get_all.len());
        assert_eq!(&roles[0].name, &get_all[0].name);
        assert_eq!(&roles[1].name, &get_all[1].name);
    }

    #[test]
    fn should_return_empty_if_no_matching_names() {
        let roles =
            vec![Role { name: "RoleOne".to_string(), permissions: vec![] }, Role { name: "RoleTwo".to_string(), permissions: vec![] }];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let get_by_name = provider.get_by_name(&vec![]);
        assert!(get_by_name.is_empty());
    }

    #[test]
    fn should_return_role_by_name() {
        let roles =
            vec![Role { name: "RoleOne".to_string(), permissions: vec![] }, Role { name: "RoleTwo".to_string(), permissions: vec![] }];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let get_by_name = provider.get_by_name(&vec!["RoleOne".to_owned()]);
        assert!(!get_by_name.is_empty());
        assert_eq!(1, get_by_name.len());
        assert_eq!("RoleOne", &get_by_name[0].name);
    }
}

#[cfg(test)]
mod test_auth_context {

    use super::*;

    #[test]
    fn service_should_be_send_and_sync() {
        let provider = super::InMemoryRolesProvider::new(vec![]);
        let auth_service = super::AuthService { roles_provider: provider };

        call_me_with_send_and_sync(auth_service);
    }

    fn call_me_with_send_and_sync<T: Send + Sync>(_: T) {}

    #[test]
    fn should_be_authenticated() {
        let provider = super::InMemoryRolesProvider::new(vec![]);
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec![] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.is_authenticated().is_ok());
    }

    #[test]
    fn should_be_not_authenticated() {
        let provider = super::InMemoryRolesProvider::new(vec![]);
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "".to_string(), roles: vec![] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.is_authenticated().is_err());
    }

    #[test]
    fn should_be_not_authenticated_even_if_has_role() {
        let provider = super::InMemoryRolesProvider::new(vec![]);
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "".to_string(), roles: vec!["ADMIN".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_role("ADMIN").is_err());
    }

    #[test]
    fn should_have_role() {
        let provider = super::InMemoryRolesProvider::new(vec![]);
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["ADMIN".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_role("ADMIN").is_ok());
    }

    #[test]
    fn should_have_role_2() {
        let provider = super::InMemoryRolesProvider::new(vec![]);
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["ADMIN".to_string(), "USER".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_role("USER").is_ok());
    }

    #[test]
    fn should_have_role_chained() {
        let provider = super::InMemoryRolesProvider::new(vec![]);
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["ADMIN".to_string(), "USER".to_string()] };
        let auth = auth_service.auth(user);
        assert!(auth.has_role("USER").and_then(|auth| auth.has_role("USER")).is_ok());
    }

    #[test]
    fn should_not_have_role() {
        let provider = super::InMemoryRolesProvider::new(vec![]);
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["ADMIN".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_role("USER").is_err());
    }

    #[test]
    fn should_have_any_role() {
        let provider = super::InMemoryRolesProvider::new(vec![]);
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["ADMIN".to_string(), "USER".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_any_role(&["USER", "FRIEND"]).is_ok());
    }

    #[test]
    fn should_not_have_any_role() {
        let provider = super::InMemoryRolesProvider::new(vec![]);
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["ADMIN".to_string(), "OWNER".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_any_role(&["USER", "FRIEND"]).is_err());
    }

    #[test]
    fn should_have_all_roles() {
        let provider = super::InMemoryRolesProvider::new(vec![]);
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["ADMIN".to_string(), "USER".to_string(), "FRIEND".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_all_roles(&["USER", "FRIEND"]).is_ok());
    }

    #[test]
    fn should_not_have_all_roles() {
        let provider = super::InMemoryRolesProvider::new(vec![]);
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["ADMIN".to_string(), "USER".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_all_roles(&["USER", "FRIEND"]).is_err());
    }

    #[test]
    fn should_be_not_authenticated_even_if_has_permission() {
        let roles = vec![Role { name: "ADMIN".to_string(), permissions: vec!["delete".to_string()] }];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "".to_string(), roles: vec!["ADMIN".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_permission("delete").is_err());
    }

    #[test]
    fn should_have_permission() {
        let roles = vec![
            Role { name: "ADMIN".to_string(), permissions: vec!["delete".to_string()] },
            Role { name: "OWNER".to_string(), permissions: vec!["create".to_string()] },
        ];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["ADMIN".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_permission("delete").is_ok());
    }

    #[test]
    fn should_have_permission_2() {
        let roles = vec![
            Role { name: "ADMIN".to_string(), permissions: vec!["delete".to_string()] },
            Role { name: "OWNER".to_string(), permissions: vec!["delete".to_string()] },
        ];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["ADMIN".to_string(), "OWNER".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_permission("delete").is_ok());
    }

    #[test]
    fn should_not_have_permission() {
        let roles = vec![
            Role { name: "ADMIN".to_string(), permissions: vec!["delete".to_string()] },
            Role { name: "OWNER".to_string(), permissions: vec!["delete".to_string()] },
        ];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["USER".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_permission("delete").is_err());
    }

    #[test]
    fn should_have_any_permission() {
        let roles = vec![
            Role { name: "ADMIN".to_string(), permissions: vec!["superDelete".to_string()] },
            Role { name: "OWNER".to_string(), permissions: vec!["delete".to_string()] },
        ];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["USER".to_string(), "ADMIN".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_any_permission(&["delete", "superDelete"]).is_ok());
    }

    #[test]
    fn should_not_have_any_permission() {
        let roles = vec![
            Role { name: "ADMIN".to_string(), permissions: vec!["delete".to_string(), "superDelete".to_string()] },
            Role { name: "OWNER".to_string(), permissions: vec!["delete".to_string()] },
        ];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["USER".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_any_permission(&["delete", "superAdmin"]).is_err());
    }

    #[test]
    fn should_have_all_permissions() {
        let roles = vec![
            Role { name: "ADMIN".to_string(), permissions: vec!["superDelete".to_string()] },
            Role { name: "OWNER".to_string(), permissions: vec!["delete".to_string()] },
            Role { name: "USER".to_string(), permissions: vec!["delete".to_string()] },
        ];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["USER".to_string(), "ADMIN".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_all_permissions(&["delete", "superDelete"]).is_ok());
    }

    #[test]
    fn should_not_have_all_permissions() {
        let roles = vec![
            Role { name: "ADMIN".to_string(), permissions: vec!["superDelete".to_string()] },
            Role { name: "OWNER".to_string(), permissions: vec!["delete".to_string()] },
        ];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["USER".to_string(), "ADMIN".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_all_permissions(&["delete", "superDelete"]).is_err());
    }

    #[test]
    fn should_be_the_owner() {
        let roles = vec![];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["USER".to_string(), "ADMIN".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.is_owner(&Ownable { owner_id: 0 }).is_ok());
    }

    #[test]
    fn should_not_be_the_owner() {
        let roles = vec![];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["USER".to_string(), "ADMIN".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.is_owner(&Ownable { owner_id: 1 }).is_err());
    }

    #[test]
    fn should_be_allowed_if_not_the_owner_but_has_role() {
        let roles = vec![Role { name: "ROLE_1".to_string(), permissions: vec!["access_1".to_string()] }];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["ROLE_1".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.is_owner_or_has_role(&Ownable { owner_id: 1 }, "ROLE_1").is_ok());
    }

    #[test]
    fn should_be_allowed_if_the_owner_but_not_has_role() {
        let roles = vec![Role { name: "ROLE_1".to_string(), permissions: vec!["access_1".to_string()] }];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["ROLE_1".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.is_owner_or_has_role(&Ownable { owner_id: 0 }, "ROLE_2").is_ok());
    }

    #[test]
    fn should_not_be_allowed_if_not_the_owner_and_not_has_role() {
        let roles = vec![Role { name: "ROLE_1".to_string(), permissions: vec!["access_1".to_string()] }];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["ROLE_1".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.is_owner_or_has_role(&Ownable { owner_id: 1 }, "ROLE_2").is_err());
    }

    #[test]
    fn should_be_allowed_if_not_the_owner_but_has_permission() {
        let roles = vec![Role { name: "ROLE_1".to_string(), permissions: vec!["access_1".to_string()] }];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["ROLE_1".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.is_owner_or_has_permission(&Ownable { owner_id: 1 }, "access_1").is_ok());
    }

    #[test]
    fn should_be_allowed_if_the_owner_but_not_has_permission() {
        let roles = vec![Role { name: "ROLE_1".to_string(), permissions: vec!["access_1".to_string()] }];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["ROLE_1".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.is_owner_or_has_permission(&Ownable { owner_id: 0 }, "access_2").is_ok());
    }

    #[test]
    fn should_not_be_allowed_if_not_the_owner_and_not_has_permission() {
        let roles = vec![Role { name: "ROLE_1".to_string(), permissions: vec!["access_1".to_string()] }];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["ROLE_1".to_string()] };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.is_owner_or_has_permission(&Ownable { owner_id: 1 }, "access_2").is_err());
    }

    #[test]
    fn should_return_true_if_all_matches() -> Result<(), LightSpeedError> {
        let roles = vec![Role { name: "ROLE_1".to_string(), permissions: vec!["access_1".to_string()] }];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["ROLE_1".to_string(), "ROLE_2".to_string()] };
        let auth_context = auth_service.auth(user);

        assert!(auth_context.has_role("ROLE_1")?.has_any_role(&vec!["ROLE_1", "ROLE_3"]).is_ok());

        assert!(auth_context.has_role("ROLE_3").and_then(|auth| auth.has_any_role(&vec!["ROLE_1", "ROLE_3"])).is_err());

        assert!(auth_context.has_role("ROLE_1").and_then(|auth| auth.has_all_roles(&vec!["ROLE_1", "ROLE_3"])).is_err());
        Ok(())
    }

    #[test]
    fn should_return_true_if_any_matches() -> Result<(), LightSpeedError> {
        let roles = vec![Role { name: "ROLE_1".to_string(), permissions: vec!["access_1".to_string()] }];
        let provider = super::InMemoryRolesProvider::new(roles.clone());
        let auth_service = super::AuthService { roles_provider: provider };
        let user = Auth { id: 0, username: "name".to_string(), roles: vec!["ROLE_1".to_string(), "ROLE_2".to_string()] };
        let auth_context = auth_service.auth(user);

        assert!(auth_context
            .has_role("ROLE_1")
            .or_else(|_err| auth_context.has_any_role(&vec!["ROLE_1", "ROLE_3"])?.has_role("ROLE_1"))
            .is_ok());

        assert!(auth_context.has_role("ROLE_3").or_else(|_err| auth_context.has_any_role(&vec!["ROLE_1", "ROLE_3"])).is_ok());

        assert!(auth_context.has_role("ROLE_1").or_else(|_err| auth_context.has_all_roles(&vec!["ROLE_1", "ROLE_3"])).is_ok());

        assert!(auth_context.has_role("ROLE_3").or_else(|_err| auth_context.has_all_roles(&vec!["ROLE_1", "ROLE_3"])).is_err());
        Ok(())
    }

    struct Ownable {
        owner_id: i64,
    }

    impl Owned for Ownable {
        fn get_owner_id(&self) -> i64 {
            return self.owner_id;
        }
    }
}
