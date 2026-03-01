use serde::{Deserialize, Serialize};

/// Role-Based Access Control roles matching the database schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    GUEST,
    USER,
    TRAINER,
    SUPERADMIN,
}

impl Role {
    /// Parse a role from its string representation (case-insensitive).
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "GUEST" => Role::GUEST,
            "TRAINER" => Role::TRAINER,
            "SUPERADMIN" | "SUPER_ADMIN" | "ADMIN" => Role::SUPERADMIN,
            _ => Role::USER,
        }
    }

    /// Return the string representation used in the database.
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::GUEST => "GUEST",
            Role::USER => "USER",
            Role::TRAINER => "TRAINER",
            Role::SUPERADMIN => "SUPERADMIN",
        }
    }

    /// Returns a numeric privilege level for comparison.
    /// Higher values indicate more privileges.
    pub fn level(&self) -> u8 {
        match self {
            Role::GUEST => 0,
            Role::USER => 1,
            Role::TRAINER => 2,
            Role::SUPERADMIN => 3,
        }
    }

    /// Returns true if this role has at least the privileges of `required`.
    pub fn has_access(&self, required: &Role) -> bool {
        self.level() >= required.level()
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// User model matching the database `users` table.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub created_at: String,
}

/// BrainNode model matching the database `brain_nodes_v2` table.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BrainNodeV2 {
    pub id: String,
    pub fact: String,
    pub source_url: Option<String>,
    pub vector_id: Option<String>,
    pub category: String,
    pub created_at: String,
}

/// Connection model matching the database `connections` table.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Connection {
    pub id: String,
    pub from_node_id: String,
    pub to_node_id: String,
    pub strength: f64,
}

/// ActionLog model matching the database `action_logs` table.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ActionLog {
    pub id: String,
    pub user_id: Option<String>,
    pub action: String,
    pub details: String,
    pub severity: String,
    pub timestamp: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_from_str() {
        assert_eq!(Role::from_str("GUEST"), Role::GUEST);
        assert_eq!(Role::from_str("user"), Role::USER);
        assert_eq!(Role::from_str("trainer"), Role::TRAINER);
        assert_eq!(Role::from_str("SUPERADMIN"), Role::SUPERADMIN);
        assert_eq!(Role::from_str("super_admin"), Role::SUPERADMIN);
        assert_eq!(Role::from_str("admin"), Role::SUPERADMIN);
        assert_eq!(Role::from_str("unknown"), Role::USER);
    }

    #[test]
    fn test_role_level_ordering() {
        assert!(Role::SUPERADMIN.level() > Role::TRAINER.level());
        assert!(Role::TRAINER.level() > Role::USER.level());
        assert!(Role::USER.level() > Role::GUEST.level());
    }

    #[test]
    fn test_role_has_access() {
        assert!(Role::SUPERADMIN.has_access(&Role::GUEST));
        assert!(Role::SUPERADMIN.has_access(&Role::USER));
        assert!(Role::SUPERADMIN.has_access(&Role::TRAINER));
        assert!(Role::SUPERADMIN.has_access(&Role::SUPERADMIN));
        assert!(Role::TRAINER.has_access(&Role::USER));
        assert!(!Role::USER.has_access(&Role::TRAINER));
        assert!(!Role::GUEST.has_access(&Role::USER));
    }
}
