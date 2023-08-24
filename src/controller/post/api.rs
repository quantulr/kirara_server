use std::sync::Arc;

use axum::extract::{Query, State};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::StatusCode;
use axum::{Json, TypedHeader};
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
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
    let page = pagination.page;
    let per_page = pagination.per_page;
    let paginator = posts::Entity::find()
        .order_by_desc(posts::Column::CreatedAt)
        .paginate(conn, per_page);
    let post_list = paginator.fetch_page(page - 1).await;
    let num_items_and_pages = paginator.num_items_and_pages().await;
    match (post_list, num_items_and_pages) {
        (Ok(post_list), Ok(item_page_count)) => {
            // let media_list = media::Entity::find().filter(media::Column::PostId.eq());
            let mut post_list_with_media: Vec<PostResponse> = Vec::new();
            for post in &post_list {
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
            let has_next = page < item_page_count.number_of_pages;
            let image_history_response = PostListResponse {
                items: post_list_with_media,
                total: item_page_count.number_of_items,
                total_pages: item_page_count.number_of_pages,
                has_next,
            };
            Ok(Json(image_history_response))
        }
        _ => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message":"列表查询失败"})),
        )),
    }
}
