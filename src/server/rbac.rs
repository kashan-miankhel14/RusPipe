/// M9: RBAC — role-based access control for API routes.
/// Roles: viewer < operator < admin
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use axum::extract::Request;
use serde_json::json;

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub name: String,
    pub role: String,
}

/// Role hierarchy check: does `actual` satisfy `required`?
pub fn role_allows(actual: &str, required: &str) -> bool {
    let rank = |r: &str| match r {
        "admin"    => 3,
        "operator" => 2,
        "viewer"   => 1,
        _          => 0,
    };
    rank(actual) >= rank(required)
}

/// Axum middleware: extract role from `X-Role` + `X-User` headers (demo auth).
pub async fn auth_middleware(mut req: Request, next: Next) -> Response {
    let role = req
        .headers()
        .get("x-role")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("viewer")
        .to_string();
    let name = req
        .headers()
        .get("x-user")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anonymous")
        .to_string();

    req.extensions_mut().insert(AuthUser { name, role });
    next.run(req).await
}

/// Extractor so handlers can receive AuthUser as a parameter.
#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for AuthUser {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "missing auth"})),
                )
            })
    }
}
