use std::sync::Arc;

use axum::extract::State;
use axum::http::{header, Request, StatusCode};
use axum::Json;
use axum::middleware::Next;
use axum::response::IntoResponse;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{json, Value};

use crate::AppState;
use crate::controller::user::response::Claims;
use crate::entities::users;

// 给定一个字符串，与一个数组中的正则表达式进行匹配，如果匹配成功，则返回true，否则返回false
fn match_path_whitelist(str: &str) -> bool {
    let regexps = vec![
        r"^/image/\d{4}/\d{2}/\d{2}/\w+$",
        r"^/user/login$",
        r"^/user/register$",
    ];
    for regex in regexps {
        let re = regex::Regex::new(regex).unwrap();
        if re.is_match(str) {
            return true;
        }
    }
    false
}


pub async fn auth<B>(
    State(state): State<Arc<AppState>>,
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let skip = match_path_whitelist(req.uri().path());
    if skip {
        return Ok(next.run(req).await);
    }

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
    let token = match auth_str {
        Some(token) => token,
        None => {
            return Err((StatusCode::UNAUTHORIZED, Json(json!({"message":"未登录"}))));
        }
    };
    let token_data = match jsonwebtoken::decode::<Claims>(
        &token,
        &DecodingKey::from_secret("secret".as_ref()),
        &Validation::new(Algorithm::HS512),
    ) {
        Ok(data) => data,
        Err(err) => {
            println!("{}", err);
            return Err((StatusCode::UNAUTHORIZED, Json(json!({"message":"未登录"}))));
        }
    };
    let username = token_data.claims.username;
    let conn = &state.conn;
    let user_model = users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .one(conn)
        .await;
    let _user = match user_model {
        Err(_) => {
            return Err((StatusCode::UNAUTHORIZED, Json(json!({"message":"未登录"}))));
        }
        _ => {}
    };
    Ok(next.run(req).await)
}
