use actix_session::Session;
use actix_web::HttpRequest;
use serde_json::json;

use crate::models::Role;

/// Extract the effective Role for the current request from session or JWT bearer token.
///
/// Priority: session role → bearer token claims → GUEST fallback.
pub fn extract_role(session: &Session, http_req: &HttpRequest) -> Role {
    // 1. Try session first
    if let Ok(Some(true)) = session.get::<bool>("logged_in") {
        if let Ok(Some(role_str)) = session.get::<String>("role") {
            return Role::from_str(&role_str);
        }
        // Logged in but no explicit role stored — default to USER
        return Role::USER;
    }

    // 2. Try bearer token
    if let Some(claims) = crate::auth::extract_bearer_claims(http_req) {
        // Look up role from token username
        let role_str = if claims.is_admin {
            "SUPERADMIN"
        } else {
            "USER"
        };
        return Role::from_str(role_str);
    }

    // 3. Fallback: unauthenticated
    Role::GUEST
}

/// Check whether the current request has at least the required role.
/// Returns `Ok(role)` if access is granted, or `Err(HttpResponse)` if denied.
pub fn require_role(
    session: &Session,
    http_req: &HttpRequest,
    required: Role,
) -> Result<Role, actix_web::HttpResponse> {
    let current_role = extract_role(session, http_req);

    // The hardcoded root admin always has full access
    let is_root = session
        .get::<String>("username")
        .ok()
        .flatten()
        .map(|u| u == crate::auth::ROOT_ADMIN_USERNAME)
        .unwrap_or(false);

    if is_root {
        return Ok(Role::SUPERADMIN);
    }

    if current_role.has_access(&required) {
        Ok(current_role)
    } else {
        Err(actix_web::HttpResponse::Forbidden().json(json!({
            "error": "Insufficient privileges",
            "required_role": required.as_str(),
            "current_role": current_role.as_str()
        })))
    }
}

/// Helper: require at least USER role (authenticated user).
pub fn require_user(
    session: &Session,
    http_req: &HttpRequest,
) -> Result<Role, actix_web::HttpResponse> {
    require_role(session, http_req, Role::USER)
}

/// Helper: require at least TRAINER role.
pub fn require_trainer(
    session: &Session,
    http_req: &HttpRequest,
) -> Result<Role, actix_web::HttpResponse> {
    require_role(session, http_req, Role::TRAINER)
}

/// Helper: require SUPERADMIN role.
pub fn require_admin(
    session: &Session,
    http_req: &HttpRequest,
) -> Result<Role, actix_web::HttpResponse> {
    require_role(session, http_req, Role::SUPERADMIN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_hierarchy_enforcement() {
        // Verify the static role hierarchy is correct
        assert!(Role::SUPERADMIN.has_access(&Role::TRAINER));
        assert!(Role::TRAINER.has_access(&Role::USER));
        assert!(Role::USER.has_access(&Role::GUEST));
        assert!(!Role::GUEST.has_access(&Role::USER));
        assert!(!Role::USER.has_access(&Role::TRAINER));
        assert!(!Role::TRAINER.has_access(&Role::SUPERADMIN));
    }
}
