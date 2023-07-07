use std::sync::Arc;

use axum::extract::State;
use axum::http::{header, Method, Request, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::Json;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{json, Value};

use crate::controller::user::response::Claims;
use crate::entities::users;
use crate::AppState;

fn should_skip_auth(str: &str, method: &Method) -> bool {
    let regexps = vec![
        (r"^/$", Method::GET),
        (r"^/favicon.ico$", Method::GET),
        (r"^/user/login$", Method::POST),
        (r"^/user/register$", Method::POST),
        (r"^/v/s/\d{4}/\d{2}/\d{2}/\w+\.\w+$", Method::GET),
        (r"^/v/p/\d{4}/\d{2}/\d{2}/\w+\.\w+$", Method::GET),
        (r"^/image/\d{4}/\d{2}/\d{2}/\w+\.\w+$", Method::GET),
        (
            r"^/image/thumbnail/\d{4}/\d{2}/\d{2}/\w+\.\w+$",
            Method::GET,
        ),
    ];
    for (regex, m) in regexps {
        let re = regex::Regex::new(regex).unwrap();
        if re.is_match(str) && m == *method {
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
    // 如果请求的路径在白名单中，则直接跳过认证
    let skip = should_skip_auth(req.uri().path(), req.method());
    if skip {
        return Ok(next.run(req).await);
    }

    // 从请求头中获取token
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

    // 如果token不存在，则返回未登录的错误
    let token = match auth_str {
        Some(token) => token,
        None => {
            return Err((StatusCode::UNAUTHORIZED, Json(json!({"message":"未登录"}))));
        }
    };

    // 如果token存在，则进行解析
    let token_data = match jsonwebtoken::decode::<Claims>(
        &token,
        &DecodingKey::from_secret(&state.jwt_secret.as_ref()),
        &Validation::new(Algorithm::HS512),
    ) {
        Ok(data) => data,
        Err(_err) => {
            return Err((StatusCode::UNAUTHORIZED, Json(json!({"message":"未登录"}))));
        }
    };

    // 如果token过期则返回未登录的错误
    if token_data.claims.exp < chrono::Local::now().timestamp_millis() as usize {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"message":"登录已过期，请重新登录"})),
        ));
    }

    let username = token_data.claims.username; // 获取用户名
    let conn = &state.conn; // 获取数据库连接

    // 从数据库中查找用户
    let user_model = users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .one(conn)
        .await;

    // 如果用户不存在，则返回未登录的错误
    match user_model {
        Ok(Some(_user)) => Ok(next.run(req).await),
        _ => {
            return Err((StatusCode::UNAUTHORIZED, Json(json!({"message":"未登录"}))));
        }
    }
}
