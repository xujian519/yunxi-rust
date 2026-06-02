use super::{Permission, PermissionAware, PermissionCheckResult, PermissionLevel, Role};
use std::collections::{HashMap, HashSet};

pub struct PermissionChecker {
    roles: HashMap<String, Role>,
    user_roles: HashMap<String, HashSet<String>>,
    fallback: PermissionLevel,
}

impl PermissionChecker {
    pub fn new() -> Self {
        Self {
            roles: HashMap::new(),
            user_roles: HashMap::new(),
            fallback: PermissionLevel::Deny,
        }
    }

    pub fn with_fallback(mut self, fallback: PermissionLevel) -> Self {
        self.fallback = fallback;
        self
    }

    pub fn add_role(&mut self, role: Role) {
        self.roles.insert(role.name.clone(), role);
    }

    pub fn remove_role(&mut self, name: &str) {
        self.roles.remove(name);
    }

    pub fn get_role(&self, name: &str) -> Option<&Role> {
        self.roles.get(name)
    }

    pub fn add_user_role(&mut self, user: String, role: String) {
        self.user_roles
            .entry(user)
            .or_insert_with(HashSet::new)
            .insert(role);
    }

    pub fn remove_user_role(&mut self, user: &str, role: &str) {
        if let Some(roles) = self.user_roles.get_mut(user) {
            roles.remove(role);
        }
    }

    pub fn check(&self, user: &str, permission: &str) -> PermissionCheckResult {
        if let Some(role_names) = self.user_roles.get(user) {
            for role_name in role_names {
                if let Some(result) = self.check_role(role_name, permission) {
                    return result;
                }
            }
        }
        match self.fallback {
            PermissionLevel::Allow => PermissionCheckResult::Allowed,
            PermissionLevel::Deny => PermissionCheckResult::Denied,
            PermissionLevel::Restricted => PermissionCheckResult::Restricted,
        }
    }

    pub fn check_role(&self, role_name: &str, permission: &str) -> Option<PermissionCheckResult> {
        let role = self.roles.get(role_name)?;

        if let Some(level) = role.get_permission_level(permission) {
            return Some(match level {
                PermissionLevel::Allow => PermissionCheckResult::Allowed,
                PermissionLevel::Deny => PermissionCheckResult::Denied,
                PermissionLevel::Restricted => PermissionCheckResult::Restricted,
            });
        }

        for inherited_role in &role.inherits {
            if let Some(result) = self.check_role(inherited_role, permission) {
                return Some(result);
            }
        }

        None
    }

    pub fn has_role(&self, user: &str, role: &str) -> bool {
        self.user_roles
            .get(user)
            .map(|roles| roles.contains(role))
            .unwrap_or(false)
    }

    pub fn user_has_permission(&self, user: &str, permission: &str) -> bool {
        self.check(user, permission).is_allowed()
    }

    pub fn add_permission_to_role(&mut self, role: &str, permission: Permission) -> bool {
        if let Some(role) = self.roles.get_mut(role) {
            role.add_permission(permission);
            true
        } else {
            false
        }
    }

    pub fn remove_permission_from_role(&mut self, role: &str, permission: &str) -> bool {
        if let Some(role) = self.roles.get_mut(role) {
            role.remove_permission(permission);
            true
        } else {
            false
        }
    }

    pub fn list_roles(&self) -> Vec<String> {
        self.roles.keys().cloned().collect()
    }

    pub fn list_user_roles(&self, user: &str) -> Vec<String> {
        self.user_roles
            .get(user)
            .map(|roles| roles.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn list_all_permissions(&self) -> Vec<Permission> {
        let mut permissions = HashSet::new();

        for role in self.roles.values() {
            permissions.extend(role.permissions.iter().cloned());
        }

        permissions.into_iter().collect()
    }
}

impl Default for PermissionChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionAware for PermissionChecker {
    fn check_permission(&self, permission: &str) -> PermissionCheckResult {
        self.check("default", permission)
    }

    fn has_role(&self, role: &str) -> bool {
        self.roles.contains_key(role)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_checker() -> PermissionChecker {
        let mut checker = PermissionChecker::new();

        let admin =
            Role::new("admin".to_string(), "Administrator".to_string()).with_permissions(vec![
                Permission::allow("*".to_string(), "All permissions".to_string()),
            ]);

        let user = Role::new("user".to_string(), "Regular user".to_string())
            .with_permissions(vec![
                Permission::allow("read".to_string(), "Can read".to_string()),
                Permission::deny("delete".to_string(), "Cannot delete".to_string()),
            ])
            .with_inherits(vec!["guest".to_string()]);

        let guest =
            Role::new("guest".to_string(), "Guest user".to_string()).with_permissions(vec![
                Permission::allow("view".to_string(), "Can view".to_string()),
            ]);

        checker.add_role(admin);
        checker.add_role(user);
        checker.add_role(guest);

        checker.add_user_role("alice".to_string(), "admin".to_string());
        checker.add_user_role("bob".to_string(), "user".to_string());
        checker.add_user_role("charlie".to_string(), "guest".to_string());

        checker
    }

    #[test]
    fn test_check_permission() {
        let checker = setup_checker();

        assert!(checker.user_has_permission("bob", "read"));
        assert!(!checker.user_has_permission("bob", "delete"));
        assert!(checker.user_has_permission("bob", "view"));
        assert!(checker.user_has_permission("charlie", "view"));
    }

    #[test]
    fn test_has_role() {
        let checker = setup_checker();

        assert!(checker.has_role("bob", "user"));
        assert!(!checker.has_role("bob", "admin"));
    }

    #[test]
    fn test_permission_inheritance() {
        let checker = setup_checker();

        assert!(checker.user_has_permission("bob", "view"));
    }

    #[test]
    fn test_add_remove_permission() {
        let mut checker = PermissionChecker::new();

        let role = Role::new("test".to_string(), "Test role".to_string());
        let user_role = Role::new("user".to_string(), "User role".to_string());
        checker.add_role(role);
        checker.add_role(user_role);
        checker.add_user_role("testuser".to_string(), "user".to_string());

        checker.add_permission_to_role(
            "user",
            Permission::allow("new_perm".to_string(), "".to_string()),
        );
        assert!(checker.user_has_permission("testuser", "new_perm"));

        checker.remove_permission_from_role("user", "new_perm");
        assert!(!checker.user_has_permission("testuser", "new_perm"));
    }

    #[test]
    fn test_wildcard_permission() {
        let checker = setup_checker();
        assert!(checker.user_has_permission("alice", "*"));
    }
}
