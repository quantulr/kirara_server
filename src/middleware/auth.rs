use std::sync::Arc;

use axum::extract::State;
use axum::http::{header, Request, StatusCode};
use axum::Json;
use axum::middleware::Next;
use axum::response::IntoResponse;
use serde_json::{json, Value};

use crate::AppState;

pub async fn auth<B>(
    State(_state): State<Arc<AppState>>,
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let auth_str = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|auth_str| auth_str.to_str().ok())
        .and_then(|auth_value| {
            if auth_value.starts_with("Bearer ") {
                Some(auth_value[7..].to_owned())
            } else {
                None
            }
        });
    let _token = match auth_str {
        Some(token) => token,
        None => {
            return Err((StatusCode::UNAUTHORIZED, Json(json!({"message":"未登录"}))));
        }
    };
    Ok(next.run(req).await)
}
