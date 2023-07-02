use std::sync::Arc;

use axum::extract::State;
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::StatusCode;

use axum::{Json, TypedHeader};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{json, Value};

use crate::controller::user::request::{LoginUser, RegisterUser, UpdateUser};
use crate::controller::user::response::{Claims, LoginResponse};
use crate::entities::users;
use crate::utils::user::get_user_from_token;
use crate::AppState;

pub async fn login(
    State(state): State<Arc<AppState>>,
    form_data: Json<LoginUser>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<Value>)> {
    let conn = &state.conn;
    let username = &form_data.username;
    let user_model = users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .one(conn)
        .await;

    let user = match user_model {
        Ok(Some(user)) => {
            if user.password == form_data.password {
                user
            } else {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"message":"密码错误！"})),
                ));
            }
        }
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({"message":"用户不存在！"})),
            ));
        }
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "message": format!("{}", err) })),
            ));
        }
    };
    let header = Header::new(Algorithm::HS512);
    let timestamp = chrono::Local::now().timestamp_millis();

    // 有效期为30天
    let exp_timestamp = timestamp + 1000 * 60 * 60 * 24 * 30;

    let my_claims = Claims {
        username: user.username,
        email: Some(user.email),
        exp: exp_timestamp as usize,
    };

    let token = jsonwebtoken::encode(
        &header,
        &my_claims,
        &EncodingKey::from_secret(&state.jwt_secret.as_ref()),
    )
    .expect("生成token失败");
    let login_resp = LoginResponse { token };
    Ok(Json(login_resp))
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    form_data: Json<RegisterUser>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let conn = &state.conn;
    let username = &form_data.username;
    let password = &form_data.password;
    let email = &form_data.email;
    let nickname = &form_data.nickname;

    // 检查用户名是否存在
    let user_model = users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .one(conn)
        .await;
    match user_model {
        Ok(Some(_)) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"message":"用户名已存在！"})),
            ));
        }
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "message": format!("{}", err) })),
            ));
        }
        _ => {}
    };
    let user = users::ActiveModel {
        username: Set(username.clone()),
        password: Set(password.clone()),
        email: Set(email.clone()),
        nickname: Set(nickname.clone()),
        ..Default::default()
    };
    match users::Entity::insert(user).exec(conn).await {
        Ok(_) => Ok(Json(json!({"message":"注册成功"}))),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": format!("{}", err) })),
        )),
    }
}

pub async fn user_info(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<users::Model>, (StatusCode, Json<Value>)> {
    let conn = &state.conn;
    let jwt_secret = &state.jwt_secret;
    let token = auth.token();
    match get_user_from_token(token, jwt_secret, conn).await {
        Some(user) => Ok(Json(user)),
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"message":"未登录！"})),
        )),
    }
}

pub async fn update_user(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(user_info): Json<UpdateUser>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let token = auth.token();
    let jwt_secret = &state.jwt_secret;
    let conn = &state.conn;
    match get_user_from_token(token, jwt_secret, conn).await {
        Some(user_model) => {
            let mut user: users::ActiveModel = user_model.into();

            if let Some(nickname) = &user_info.nickname {
                user.nickname = Set(nickname.to_owned());
            }
            if let Some(avatar) = &user_info.avatar {
                user.avatar = Set(Some(avatar.to_owned()));
            }
            if let Some(gender) = &user_info.gender {
                user.gender = Set(gender.to_owned());
            }

            match user.update(conn).await {
                Ok(_u) => Ok(Json(json!("更新成功"))),
                Err(err) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "message": format!("{}", err) })),
                )),
            }
        }
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"message":"未登录！"})),
        )),
    }
}
