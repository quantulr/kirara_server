use std::sync::Arc;

use axum::{Json, TypedHeader};
use axum::extract::{Multipart, State};
use axum::headers::Authorization;
use axum::headers::authorization::Bearer;
use axum::http::StatusCode;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde_json::{json, Value};

use crate::AppState;
use crate::controller::user::response::Claims;
use crate::entities::media;

pub async fn upload_media(
    State(state): State<Arc<AppState>>,
    // TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let upload_path = &state.upload_path; // 上传路径
    let conn = &state.conn; // 数据库连接
    // let token = auth.token(); // token
    // // 从token中获取用户名
    // let username = match jsonwebtoken::decode::<Claims>(
    //     token,
    //     &DecodingKey::from_secret(&state.jwt_secret.as_ref()),
    //     &Validation::new(Algorithm::HS512),
    // ) {
    //     Ok(data) => data.claims.username,
    //     Err(err) => {
    //         return Err((
    //             StatusCode::UNAUTHORIZED,
    //             Json(json!({
    //                 "message": format!("身份验证失败！{}", err)
    //             })),
    //         ));
    //     }
    // };
    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name()
            .and_then(|name| {
                if name.eq("file") {
                    Some(name)
                } else {
                    None
                }
            }) {
            // None => {
            //     return Err((
            //         StatusCode::BAD_REQUEST,
            //         Json(json!({
            //             "message": "上传失败！"
            //         })),
            //     ));
            // }
            None => {
                continue;
            }
            _ => {}
        }
        return Ok(Json(json!({
            "message": "上传成功！"
        })));
    };
    Err((
        StatusCode::BAD_REQUEST,
        Json(json!({
            "message": "没有！"
        })),
    ))
}