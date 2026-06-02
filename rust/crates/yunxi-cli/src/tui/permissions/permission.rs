use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Permission {
    pub name: String,
    pub description: String,
    pub level: PermissionLevel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PermissionLevel {
    Allow,
    Deny,
    Restricted,
}

impl Permission {
    pub fn new(name: String, description: String, level: PermissionLevel) -> Self {
        Self {
            name,
            description,
            level,
        }
    }

    pub fn allow(name: String, description: String) -> Self {
        Self::new(name, description, PermissionLevel::Allow)
    }

    pub fn deny(name: String, description: String) -> Self {
        Self::new(name, description, PermissionLevel::Deny)
    }

    pub fn restricted(name: String, description: String) -> Self {
        Self::new(name, description, PermissionLevel::Restricted)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub description: String,
    pub permissions: HashSet<Permission>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inherits: Vec<String>,
}

impl Role {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            permissions: HashSet::new(),
            inherits: Vec::new(),
        }
    }

    pub fn with_permissions(mut self, permissions: Vec<Permission>) -> Self {
        self.permissions = permissions.into_iter().collect();
        self
    }

    pub fn with_inherits(mut self, inherits: Vec<String>) -> Self {
        self.inherits = inherits;
        self
    }

    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    pub fn remove_permission(&mut self, permission_name: &str) {
        self.permissions.retain(|p| p.name != permission_name);
    }

    pub fn has_permission(&self, permission_name: &str) -> bool {
        self.permissions.iter().any(|p| p.name == permission_name)
    }

    pub fn get_permission_level(&self, permission_name: &str) -> Option<PermissionLevel> {
        self.permissions
            .iter()
            .find(|p| p.name == permission_name)
            .map(|p| p.level)
    }
}

pub trait PermissionAware {
    fn check_permission(&self, permission: &str) -> PermissionCheckResult;
    fn has_role(&self, role: &str) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionCheckResult {
    Allowed,
    Denied,
    Restricted,
    Unknown,
}

impl PermissionCheckResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, PermissionCheckResult::Allowed)
    }

    pub fn is_denied(&self) -> bool {
        matches!(self, PermissionCheckResult::Denied)
    }

    pub fn is_restricted(&self) -> bool {
        matches!(self, PermissionCheckResult::Restricted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_creation() {
        let perm = Permission::allow("read".to_string(), "Can read files".to_string());
        assert_eq!(perm.level, PermissionLevel::Allow);
        assert_eq!(perm.name, "read");
    }

    #[test]
    fn test_role_creation() {
        let mut role = Role::new("user".to_string(), "Regular user".to_string());
        role.add_permission(Permission::allow(
            "read".to_string(),
            "Read permission".to_string(),
        ));

        assert!(role.has_permission("read"));
        assert!(!role.has_permission("write"));
    }

    #[test]
    fn test_permission_level() {
        let perm_allow = Permission::allow("test".to_string(), "".to_string());
        let perm_deny = Permission::deny("test".to_string(), "".to_string());
        let perm_restricted = Permission::restricted("test".to_string(), "".to_string());

        assert_eq!(perm_allow.level, PermissionLevel::Allow);
        assert_eq!(perm_deny.level, PermissionLevel::Deny);
        assert_eq!(perm_restricted.level, PermissionLevel::Restricted);
    }
}
