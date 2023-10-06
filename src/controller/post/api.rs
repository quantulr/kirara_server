use std::sync::Arc;

use axum::extract::{Query, State};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::StatusCode;
use axum::{Json, TypedHeader};
use sea_orm::prelude::DateTime;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    TransactionTrait,
};

use serde_json::{json, Value};

use crate::controller::post::request::{Pagination, PublishPostRequest};
use crate::controller::post::response::{PostListResponse, PostResponse};

use crate::entities::{media, posts};
use crate::utils::user::get_user_from_token;
use crate::AppState;

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

                let mut index = 0;
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
                    media.sort = Set(Some(index));
                    media.update(txn).await?;
                    index += 1;
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

pub async fn post_list(
    State(state): State<Arc<AppState>>,
    query: Query<Pagination>,
) -> Result<Json<PostListResponse>, (StatusCode, Json<Value>)> {
    // 数据库连接
    let conn = &state.conn;

    let pagination = query.0;
    let before = pagination.before;
    let after = pagination.after;
    let per_page = pagination.per_page;
    let post_vec = match (before, after) {
        (Some(before), Some(after)) => {
            posts::Entity::find()
                .filter(
                    posts::Column::CreatedAt
                        .lte(DateTime::from_timestamp_millis(before).expect("err")),
                )
                .filter(
                    posts::Column::CreatedAt
                        .gt(DateTime::from_timestamp_millis(after).expect("err")),
                )
                .order_by_desc(posts::Column::CreatedAt)
                .limit(per_page)
                .all(conn)
                .await
        }
        (None, Some(after)) => {
            posts::Entity::find()
                .filter(
                    posts::Column::CreatedAt
                        .gt(DateTime::from_timestamp_millis(after).expect("err")),
                )
                .order_by_desc(posts::Column::CreatedAt)
                .limit(per_page)
                .all(conn)
                .await
        }
        (Some(before), None) => {
            posts::Entity::find()
                .filter(
                    posts::Column::CreatedAt
                        .lte(DateTime::from_timestamp_millis(before).expect("err")),
                )
                .order_by_desc(posts::Column::CreatedAt)
                .limit(per_page)
                .all(conn)
                .await
        }
        _ => {
            posts::Entity::find()
                .order_by_desc(posts::Column::CreatedAt)
                .limit(per_page)
                .all(conn)
                .await
        }
    };

    match post_vec {
        Ok(list) => {
            let mut post_list_with_media: Vec<PostResponse> = Vec::new();
            for post in &list {
                let media_list = media::Entity::find()
                    .filter(media::Column::PostId.eq(post.id))
                    .order_by_asc(media::Column::Sort)
                    .order_by_asc(media::Column::CreatedAt)
                    .all(conn)
                    .await;
                match media_list {
                    Ok(media_resp_arr) => {
                        let post_response = PostResponse {
                            id: post.id,
                            user_id: post.user_id,
                            media_list: media_resp_arr,
                            description: post.description.to_owned(),
                            status: post.status,
                            created_at: post.created_at,
                            updated_at: post.created_at,
                        };
                        post_list_with_media.push(post_response);
                    }
                    Err(_) => {
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({"message":"列表查询失败"})),
                        ));
                    }
                };
            }
            let post_list_response = PostListResponse {
                total: 0,
                items: post_list_with_media,
            };
            Ok(Json(post_list_response))
        }
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message":"列表查询失败"})),
        )),
    }
}