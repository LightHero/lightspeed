use crate::error::LightSpeedError;
use crate::utils::current_epoch_seconds;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "poem_openapi", derive(poem_openapi::Object))]
pub struct Auth {
    pub id: i64,
    pub username: String,
    pub session_id: String,
    pub roles: Vec<String>,
    pub creation_ts_seconds: i64,
    pub expiration_ts_seconds: i64,
}

impl Auth {
    pub fn new<S: Into<String>>(
        id: i64,
        username: S,
        roles: Vec<String>,
        creation_ts_seconds: i64,
        expiration_ts_seconds: i64,
    ) -> Self {
        let session_id = format!("{id}_{creation_ts_seconds}");
        Self { id, username: username.into(), session_id, roles, creation_ts_seconds, expiration_ts_seconds }
    }
}

impl Default for Auth {
    fn default() -> Self {
        Self {
            id: -1,
            username: "".to_owned(),
            session_id: "".to_owned(),
            roles: vec![],
            creation_ts_seconds: 0,
            expiration_ts_seconds: 0,
        }
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

impl Owned for i64 {
    fn get_owner_id(&self) -> i64 {
        *self
    }
}

#[cfg(feature = "c3p0")]
impl<T: Owned + Clone + serde::ser::Serialize + Send> Owned for c3p0_common::Model<T> {
    fn get_owner_id(&self) -> i64 {
        self.data.get_owner_id()
    }
}

#[derive(Clone)]
pub struct AuthService<T: RolesProvider> {
    roles_provider: T,
    permission_roles_map: BTreeMap<String, Vec<String>>,
}

impl<T: RolesProvider> AuthService<T> {
    pub fn new(roles_provider: T) -> AuthService<T> {
        AuthService {
            permission_roles_map: AuthService::<T>::roles_map_to_permissions_map(roles_provider.fetch_all().as_ref()),
            roles_provider,
        }
    }

    pub fn auth(&self, auth: Auth) -> AuthContext {
        AuthContext { auth, permission_roles_map: &self.permission_roles_map }
    }

    /// Creates a permission_roles_map from an array of Roles
    fn roles_map_to_permissions_map(roles: &[Role]) -> BTreeMap<String, Vec<String>> {
        let mut result = BTreeMap::new();
        for role in roles {
            for permission in &role.permissions {
                result.entry(permission.to_owned()).or_insert_with(Vec::new).push(role.name.clone())
            }
        }
        result
    }
}

pub struct AuthContext<'a> {
    pub auth: Auth,
    permission_roles_map: &'a BTreeMap<String, Vec<String>>,
}

impl<'a> AuthContext<'a> {
    pub fn is_authenticated(&self) -> Result<&AuthContext, LightSpeedError> {
        if self.auth.username.is_empty() || self.auth.expiration_ts_seconds < current_epoch_seconds() {
            return Err(LightSpeedError::UnauthenticatedError {});
        };
        Ok(self)
    }

    pub fn has_role(&self, role: &str) -> Result<&AuthContext, LightSpeedError> {
        self.is_authenticated()?;
        if !self.has_role_bool(role) {
            return Err(LightSpeedError::ForbiddenError {
                message: format!("User [{}] does not have the required role [{}]", self.auth.id, role),
            });
        };
        Ok(self)
    }

    pub fn has_any_role(&self, roles: &[&str]) -> Result<&AuthContext, LightSpeedError> {
        self.is_authenticated()?;
        for role in roles {
            if self.has_role_bool(role) {
                return Ok(self);
            };
        }
        Err(LightSpeedError::ForbiddenError {
            message: format!("User [{}] does not have the required role", self.auth.id),
        })
    }

    pub fn has_all_roles(&self, roles: &[&str]) -> Result<&AuthContext, LightSpeedError> {
        self.is_authenticated()?;
        for role in roles {
            if !self.has_role_bool(role) {
                return Err(LightSpeedError::ForbiddenError {
                    message: format!("User [{}] does not have the required role [{}]", self.auth.id, role),
                });
            };
        }
        Ok(self)
    }

    pub fn has_permission(&self, permission: &str) -> Result<&AuthContext, LightSpeedError> {
        self.is_authenticated()?;

        if !self.has_permission_bool(permission) {
            return Err(LightSpeedError::ForbiddenError {
                message: format!("User [{}] does not have the required permission [{}]", self.auth.id, permission),
            });
        };
        Ok(self)
    }

    pub fn has_any_permission(&self, permissions: &[&str]) -> Result<&AuthContext, LightSpeedError> {
        self.is_authenticated()?;

        for permission in permissions {
            if self.has_permission_bool(permission) {
                return Ok(self);
            };
        }
        Err(LightSpeedError::ForbiddenError {
            message: format!("User [{}] does not have the required permission", self.auth.id),
        })
    }

    pub fn has_all_permissions(&self, permissions: &[&str]) -> Result<&AuthContext, LightSpeedError> {
        self.is_authenticated()?;
        for permission in permissions {
            if !self.has_permission_bool(permission) {
                return Err(LightSpeedError::ForbiddenError {
                    message: format!("User [{}] does not have the required permission [{}]", self.auth.id, permission),
                });
            };
        }
        Ok(self)
    }

    pub fn is_owner<T: Owned>(&self, obj: &T) -> Result<&AuthContext, LightSpeedError> {
        if self.auth.id == obj.get_owner_id() {
            Ok(self)
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
            Ok(self)
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

    pub fn is_owner_or_has_permission<T: Owned>(
        &self,
        obj: &T,
        permission: &str,
    ) -> Result<&AuthContext, LightSpeedError> {
        if (self.auth.id == obj.get_owner_id()) || self.has_permission_bool(permission) {
            Ok(self)
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
        if let Some(roles_with_permission) = self.permission_roles_map.get(permission) {
            for user_role in &self.auth.roles {
                if roles_with_permission.contains(user_role) {
                    return true;
                }
            }
        };
        false
    }
}

pub trait RolesProvider: Send + Sync + Clone {
    fn fetch_all(&self) -> Cow<[Role]>;
}

#[derive(Clone)]
pub struct InMemoryRolesProvider {
    all_roles: Arc<[Role]>,
    roles_by_name: Arc<HashMap<String, Role>>,
}

impl InMemoryRolesProvider {
    pub fn new(all_roles: Arc<[Role]>) -> InMemoryRolesProvider {
        let mut roles_by_name = HashMap::new();

        for role in all_roles.iter() {
            roles_by_name.insert(role.name.clone(), role.clone());
        }

        InMemoryRolesProvider { all_roles, roles_by_name: Arc::new(roles_by_name) }
    }
}

impl RolesProvider for InMemoryRolesProvider {
    fn fetch_all(&self) -> Cow<[Role]> {
        Cow::Borrowed(self.all_roles.as_ref())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::utils::current_epoch_seconds;

    #[test]
    fn service_should_be_send_and_sync() {
        let provider = super::InMemoryRolesProvider::new(vec![].into());
        let auth_service = super::AuthService::new(provider);

        call_me_with_send_and_sync(auth_service);
    }

    fn call_me_with_send_and_sync<T: Send + Sync>(_: T) {}

    #[test]
    fn new_auth_should_generate_session_id() {
        let auth = Auth::new(321, "name".to_string(), vec![], 124560, current_epoch_seconds() + 100);

        assert_eq!("321_124560", &auth.session_id);
    }

    #[test]
    fn should_be_authenticated() {
        let provider = super::InMemoryRolesProvider::new(vec![].into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec![],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.is_authenticated().is_ok());
    }

    #[test]
    fn should_be_not_authenticated_if_no_username() {
        let provider = super::InMemoryRolesProvider::new(vec![].into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "".to_string(),
            session_id: "".to_string(),
            roles: vec![],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);

        match auth_context.is_authenticated() {
            Err(LightSpeedError::UnauthenticatedError) => {}
            _ => panic!("Should return UnauthenticatedError if no username"),
        }
    }

    #[test]
    fn should_be_not_authenticated_if_expired() {
        let provider = super::InMemoryRolesProvider::new(vec![].into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 10,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec![],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() - 1,
        };
        let auth_context = auth_service.auth(user);

        match auth_context.is_authenticated() {
            Err(LightSpeedError::UnauthenticatedError) => {}
            _ => panic!("Should return UnauthenticatedError if expired"),
        }
    }

    #[test]
    fn should_be_not_authenticated_even_if_has_role() {
        let provider = super::InMemoryRolesProvider::new(vec![].into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "".to_string(),
            session_id: "".to_string(),
            roles: vec!["ADMIN".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_role("ADMIN").is_err());
    }

    #[test]
    fn should_have_role() {
        let provider = super::InMemoryRolesProvider::new(vec![].into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["ADMIN".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_role("ADMIN").is_ok());
    }

    #[test]
    fn should_have_role_2() {
        let provider = super::InMemoryRolesProvider::new(vec![].into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["ADMIN".to_string(), "USER".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_role("USER").is_ok());
    }

    #[test]
    fn should_have_role_chained() {
        let provider = super::InMemoryRolesProvider::new(vec![].into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["ADMIN".to_string(), "USER".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth = auth_service.auth(user);
        assert!(auth.has_role("USER").and_then(|auth| auth.has_role("USER")).is_ok());
    }

    #[test]
    fn should_not_have_role() {
        let provider = super::InMemoryRolesProvider::new(vec![].into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["ADMIN".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_role("USER").is_err());
    }

    #[test]
    fn should_have_any_role() {
        let provider = super::InMemoryRolesProvider::new(vec![].into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["ADMIN".to_string(), "USER".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_any_role(&["USER", "FRIEND"]).is_ok());
    }

    #[test]
    fn should_not_have_any_role() {
        let provider = super::InMemoryRolesProvider::new(vec![].into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["ADMIN".to_string(), "OWNER".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_any_role(&["USER", "FRIEND"]).is_err());
    }

    #[test]
    fn should_have_all_roles() {
        let provider = super::InMemoryRolesProvider::new(vec![].into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["ADMIN".to_string(), "USER".to_string(), "FRIEND".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_all_roles(&["USER", "FRIEND"]).is_ok());
    }

    #[test]
    fn should_not_have_all_roles() {
        let provider = super::InMemoryRolesProvider::new(vec![].into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["ADMIN".to_string(), "USER".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_all_roles(&["USER", "FRIEND"]).is_err());
    }

    #[test]
    fn should_be_not_authenticated_even_if_has_permission() {
        let roles = vec![Role { name: "ADMIN".to_string(), permissions: vec!["delete".to_string()] }];
        let provider = super::InMemoryRolesProvider::new(roles.into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "".to_string(),
            session_id: "".to_string(),
            roles: vec!["ADMIN".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_permission("delete").is_err());
    }

    #[test]
    fn should_have_permission() {
        let roles = vec![
            Role { name: "ADMIN".to_string(), permissions: vec!["delete".to_string()] },
            Role { name: "OWNER".to_string(), permissions: vec!["create".to_string()] },
        ];
        let provider = super::InMemoryRolesProvider::new(roles.into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["ADMIN".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_permission("delete").is_ok());
    }

    #[test]
    fn should_have_permission_2() {
        let roles = vec![
            Role { name: "ADMIN".to_string(), permissions: vec!["delete".to_string()] },
            Role { name: "OWNER".to_string(), permissions: vec!["delete".to_string()] },
        ];
        let provider = super::InMemoryRolesProvider::new(roles.into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["ADMIN".to_string(), "OWNER".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_permission("delete").is_ok());
    }

    #[test]
    fn should_not_have_permission() {
        let roles = vec![
            Role { name: "ADMIN".to_string(), permissions: vec!["delete".to_string()] },
            Role { name: "OWNER".to_string(), permissions: vec!["delete".to_string()] },
        ];
        let provider = super::InMemoryRolesProvider::new(roles.into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["USER".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_permission("delete").is_err());
    }

    #[test]
    fn should_have_any_permission() {
        let roles = vec![
            Role { name: "ADMIN".to_string(), permissions: vec!["superDelete".to_string()] },
            Role { name: "OWNER".to_string(), permissions: vec!["delete".to_string()] },
        ];
        let provider = super::InMemoryRolesProvider::new(roles.into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["USER".to_string(), "ADMIN".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_any_permission(&["delete", "superDelete"]).is_ok());
    }

    #[test]
    fn should_not_have_any_permission() {
        let roles = vec![
            Role { name: "ADMIN".to_string(), permissions: vec!["delete".to_string(), "superDelete".to_string()] },
            Role { name: "OWNER".to_string(), permissions: vec!["delete".to_string()] },
        ];
        let provider = super::InMemoryRolesProvider::new(roles.into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["USER".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
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
        let provider = super::InMemoryRolesProvider::new(roles.into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["USER".to_string(), "ADMIN".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_all_permissions(&["delete", "superDelete"]).is_ok());
    }

    #[test]
    fn should_not_have_all_permissions() {
        let roles = vec![
            Role { name: "ADMIN".to_string(), permissions: vec!["superDelete".to_string()] },
            Role { name: "OWNER".to_string(), permissions: vec!["delete".to_string()] },
        ];
        let provider = super::InMemoryRolesProvider::new(roles.into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["USER".to_string(), "ADMIN".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.has_all_permissions(&["delete", "superDelete"]).is_err());
    }

    #[test]
    fn should_be_the_owner() {
        let roles = vec![];
        let provider = super::InMemoryRolesProvider::new(roles.into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["USER".to_string(), "ADMIN".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.is_owner(&Ownable { owner_id: 0 }).is_ok());
    }

    #[test]
    fn should_not_be_the_owner() {
        let roles = vec![];
        let provider = super::InMemoryRolesProvider::new(roles.into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["USER".to_string(), "ADMIN".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.is_owner(&Ownable { owner_id: 1 }).is_err());
    }

    #[test]
    fn should_be_allowed_if_not_the_owner_but_has_role() {
        let roles = vec![Role { name: "ROLE_1".to_string(), permissions: vec!["access_1".to_string()] }];
        let provider = super::InMemoryRolesProvider::new(roles.into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["ROLE_1".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.is_owner_or_has_role(&Ownable { owner_id: 1 }, "ROLE_1").is_ok());
    }

    #[test]
    fn should_be_allowed_if_the_owner_but_not_has_role() {
        let roles = vec![Role { name: "ROLE_1".to_string(), permissions: vec!["access_1".to_string()] }];
        let provider = super::InMemoryRolesProvider::new(roles.into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["ROLE_1".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.is_owner_or_has_role(&Ownable { owner_id: 0 }, "ROLE_2").is_ok());
    }

    #[test]
    fn should_not_be_allowed_if_not_the_owner_and_not_has_role() {
        let roles = vec![Role { name: "ROLE_1".to_string(), permissions: vec!["access_1".to_string()] }];
        let provider = super::InMemoryRolesProvider::new(roles.into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["ROLE_1".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.is_owner_or_has_role(&Ownable { owner_id: 1 }, "ROLE_2").is_err());
    }

    #[test]
    fn should_be_allowed_if_not_the_owner_but_has_permission() {
        let roles = vec![Role { name: "ROLE_1".to_string(), permissions: vec!["access_1".to_string()] }];
        let provider = super::InMemoryRolesProvider::new(roles.into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["ROLE_1".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.is_owner_or_has_permission(&Ownable { owner_id: 1 }, "access_1").is_ok());
    }

    #[test]
    fn should_be_allowed_if_the_owner_but_not_has_permission() {
        let roles = vec![Role { name: "ROLE_1".to_string(), permissions: vec!["access_1".to_string()] }];
        let provider = super::InMemoryRolesProvider::new(roles.into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["ROLE_1".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.is_owner_or_has_permission(&Ownable { owner_id: 0 }, "access_2").is_ok());
    }

    #[test]
    fn should_not_be_allowed_if_not_the_owner_and_not_has_permission() {
        let roles = vec![Role { name: "ROLE_1".to_string(), permissions: vec!["access_1".to_string()] }];
        let provider = super::InMemoryRolesProvider::new(roles.into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["ROLE_1".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);
        assert!(auth_context.is_owner_or_has_permission(&Ownable { owner_id: 1 }, "access_2").is_err());
    }

    #[test]
    fn should_return_true_if_all_matches() -> Result<(), LightSpeedError> {
        let roles = vec![Role { name: "ROLE_1".to_string(), permissions: vec!["access_1".to_string()] }];
        let provider = super::InMemoryRolesProvider::new(roles.into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["ROLE_1".to_string(), "ROLE_2".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);

        assert!(auth_context.has_role("ROLE_1")?.has_any_role(&["ROLE_1", "ROLE_3"]).is_ok());

        assert!(auth_context.has_role("ROLE_3").and_then(|auth| auth.has_any_role(&["ROLE_1", "ROLE_3"])).is_err());

        assert!(auth_context.has_role("ROLE_1").and_then(|auth| auth.has_all_roles(&["ROLE_1", "ROLE_3"])).is_err());
        Ok(())
    }

    #[test]
    fn should_return_true_if_any_matches() -> Result<(), LightSpeedError> {
        let roles = vec![Role { name: "ROLE_1".to_string(), permissions: vec!["access_1".to_string()] }];
        let provider = super::InMemoryRolesProvider::new(roles.into());
        let auth_service = super::AuthService::new(provider);
        let user = Auth {
            id: 0,
            username: "name".to_string(),
            session_id: "".to_string(),
            roles: vec!["ROLE_1".to_string(), "ROLE_2".to_string()],
            creation_ts_seconds: 0,
            expiration_ts_seconds: current_epoch_seconds() + 100,
        };
        let auth_context = auth_service.auth(user);

        assert!(auth_context
            .has_role("ROLE_1")
            .or_else(|_err| auth_context.has_any_role(&["ROLE_1", "ROLE_3"])?.has_role("ROLE_1"))
            .is_ok());

        assert!(auth_context
            .has_role("ROLE_3")
            .or_else(|_err| auth_context.has_any_role(&["ROLE_1", "ROLE_3"]))
            .is_ok());

        assert!(auth_context
            .has_role("ROLE_1")
            .or_else(|_err| auth_context.has_all_roles(&["ROLE_1", "ROLE_3"]))
            .is_ok());

        assert!(auth_context
            .has_role("ROLE_3")
            .or_else(|_err| auth_context.has_all_roles(&["ROLE_1", "ROLE_3"]))
            .is_err());
        Ok(())
    }

    struct Ownable {
        owner_id: i64,
    }

    impl Owned for Ownable {
        fn get_owner_id(&self) -> i64 {
            self.owner_id
        }
    }
}
