use std::sync::Arc;

use axum::body::HttpBody;
use axum::extract::State;
use axum::http::{Error, StatusCode};
use axum::Json;
use futures_util::{TryFutureExt, TryStreamExt};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::entities::users;
use crate::AppState;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoginUser {
    username: String,
    password: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoginResponse {
    token: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    form_data: Json<LoginUser>,
) -> Result<Json<users::Model>, (StatusCode, Json<Value>)> {
    let conn = &state.conn;
    let username = &form_data.username;
    let user = users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .one(conn)
        .await
        .expect("can't find user");
    match user {
        Some(usr) => Ok(Json(usr)),
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"message":"该用户不存在"})),
        )),
    }
}

pub async fn register() -> Json<Value> {
    Json(json!({"hello": "world"}))
}

// use std::collections::HashMap;
//
// use actix_web::{get, post, web, HttpResponse, Responder, Result};
// use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
// // use sea_orm::ActiveModelTrait;
// use sea_orm::ActiveValue::Set;
// use serde::{Deserialize, Serialize};
//
// use crate::AppState;
// // use sea_orm::{}
// use crate::entities::users;
//
// #[derive(Clone, Debug, Deserialize, Serialize)]
// struct LoginUser {
//     username: String,
//     password: String,
// }
//
// #[post("/register")]
// pub async fn register(
//     data: web::Data<AppState>,
//     form_data: web::Json<users::Model>,
// ) -> impl Responder {
//     let conn = &data.conn;
//     users::ActiveModel {
//         username: Set(form_data.username.to_owned()),
//         password: Set(form_data.password.to_owned()),
//         ..Default::default()
//     }
//     .save(conn)
//     .await
//     .expect("TODO: panic message");
//     let resp: HashMap<String, String> = HashMap::new();
//     HttpResponse::Ok().json(resp)
// }
//
// #[post("/login")]
// pub async fn login(
//     data: web::Data<AppState>,
//     form_data: web::Json<LoginUser>,
// ) -> Result<HttpResponse> {
//     let conn = &data.conn;
//     let username = &form_data.username;
//     let user: Option<users::Model> = users::Entity::find()
//         .filter(users::Column::Username.eq(username))
//         .one(conn)
//         .await
//         .expect("TODO: panic message");
//
//     match user {
//         Some(usr) => Ok(HttpResponse::Ok().json(usr)),
//         None => Ok(HttpResponse::Ok().body("error")),
//     }
// }
