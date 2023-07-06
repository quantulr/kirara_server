use crate::controller::post::request::PublishPostRequest;
use crate::entities::{media, posts};
use crate::utils::user::get_user_from_token;
use crate::AppState;
use axum::extract::State;
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::StatusCode;
use axum::{Json, TypedHeader};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, TransactionTrait};
use serde_json::{json, Value};
use std::sync::Arc;

// 发布帖子
pub async fn add_post(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(form_data): Json<PublishPostRequest>,
) -> Result<Json<posts::Model>, (StatusCode, Json<Value>)> {
    // 数据库连接
    let conn = &state.conn;
    // 获取用户
    let user = get_user_from_token(auth.token(), &state.jwt_secret, conn)
        .await
        .unwrap();
    // 事务 - 发布帖子
    let ts_res = conn
        .transaction::<_, posts::Model, DbErr>(|txn| {
            Box::pin(async move {
                let post_res = posts::ActiveModel {
                    user_id: Set(user.id.to_owned()),
                    description: Set(form_data.description.to_owned()),
                    ..Default::default()
                }
                .insert(txn)
                .await;
                let post = match post_res {
                    Ok(post) => post,
                    Err(db_err) => {
                        return Err(DbErr::Custom(format!("发布帖子失败: {:?}", db_err)));
                    }
                };

                for media_id in &form_data.media_ids {
                    let media_model = match media::Entity::find_by_id(media_id.to_owned())
                        .one(txn)
                        .await
                    {
                        Ok(Some(media)) => {
                            if media.post_id.is_some() {
                                return Err(DbErr::Custom(format!(
                                    "发布帖子失败: {:?}",
                                    "media already published"
                                )));
                            }
                            media
                        }
                        Ok(None) => {
                            return Err(DbErr::Custom(format!(
                                "发布帖子失败: {:?}",
                                "media not found"
                            )));
                        }
                        Err(db_err) => {
                            return Err(DbErr::Custom(format!("发布帖子失败: {:?}", db_err)));
                        }
                    };
                    let mut media: media::ActiveModel = media_model.into();
                    media.post_id = Set(Some(post.id.to_owned()));
                    media.update(txn).await?;
                }
                Ok(post)
            })
        })
        .await;
    match ts_res {
        Ok(post) => Ok(Json(post)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("发布帖子失败: {:?}", err) })),
        )),
    }
}
