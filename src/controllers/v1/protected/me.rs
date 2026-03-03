use axum::Json;
use axum::extract::Extension;
use libs::jwt::Claims;
use serde_json::json;

pub async fn me(Extension(claims): Extension<Claims>) -> Json<serde_json::Value> {
    Json(json!({
        "sub": claims.sub,
        "roles": claims.roles,
        "policies": claims.policies,
    }))
}
