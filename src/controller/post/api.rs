use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::StatusCode;

use axum::{Json, TypedHeader};

use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, TransactionTrait,
};

use serde_json::{json, Value};

use crate::controller::post::request::{Pagination, PostSearchParams, PublishPostRequest};
use crate::controller::post::response::{
    PostListResponse, PostResponse, PostSearchResult, PostSearchResults,
};

use crate::entities::prelude::Users;
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
            Json(json!({ "message": format!("发布帖子失败: {:?}", err) })),
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
    let total = match posts::Entity::find().count(conn).await {
        Ok(count) => count,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message":"列表查询失败"})),
            ));
        }
    };
    let post_vec = match (before, after) {
        (Some(before), Some(after)) => {
            posts::Entity::find()
                .filter(posts::Column::Id.gte(after))
                .filter(posts::Column::Id.lte(before))
                .order_by_desc(posts::Column::Id)
                .all(conn)
                .await
        }
        (None, Some(after)) => {
            posts::Entity::find()
                .filter(posts::Column::Id.gte(after))
                .limit(per_page)
                .order_by_desc(posts::Column::Id)
                .all(conn)
                .await
        }
        (Some(before), None) => {
            posts::Entity::find()
                .filter(posts::Column::Id.lte(before))
                .limit(per_page)
                .order_by_desc(posts::Column::Id)
                .all(conn)
                .await
        }
        _ => {
            posts::Entity::find()
                .order_by_desc(posts::Column::Id)
                .limit(per_page)
                .all(conn)
                .await
        }
    };

    match post_vec {
        Ok(list) => {
            let mut post_list_with_media: Vec<PostResponse> = Vec::new();
            for post in &list {
                let user = match post.find_related(Users).one(conn).await {
                    Ok(Some(user)) => user,
                    _ => {
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({"message" : "帖子查询失败"})),
                        ));
                    }
                };
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
                            nickname: user.nickname,
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
            let mut next_post_id: Option<i32> = None;
            let mut prev_post_id: Option<i32> = None;
            match &post_list_with_media.first() {
                Some(first_post) => {
                    match posts::Entity::find()
                        .filter(posts::Column::Id.gt(first_post.id))
                        .order_by_asc(posts::Column::Id)
                        .one(conn)
                        .await
                    {
                        Ok(Some(first_post)) => {
                            prev_post_id = Some(first_post.id);
                        }
                        Ok(None) => {}
                        Err(_) => {
                            return Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(json!({"message":"列表查询失败"})),
                            ));
                        }
                    }
                }
                None => {}
            }
            match &post_list_with_media.last() {
                Some(last_post) => {
                    match posts::Entity::find()
                        .filter(posts::Column::Id.lt(last_post.id))
                        .order_by_desc(posts::Column::Id)
                        .one(conn)
                        .await
                    {
                        Ok(Some(next_post)) => {
                            next_post_id = Some(next_post.id);
                        }
                        Ok(None) => {}
                        Err(_) => {
                            return Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(json!({"message":"列表查询失败"})),
                            ));
                        }
                    }
                }
                None => {}
            };
            let post_list_response = PostListResponse {
                total,
                items: post_list_with_media,
                prev: prev_post_id,
                next: next_post_id,
            };
            Ok(Json(post_list_response))
        }
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message":"列表查询失败"})),
        )),
    }
}

// 帖子详情
pub async fn post_detail(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<i32>,
) -> Result<Json<PostResponse>, (StatusCode, Json<Value>)> {
    // 数据库连接
    let conn = &state.conn;
    match posts::Entity::find_by_id(post_id).one(conn).await {
        Ok(Some(post)) => {
            let user = match post.find_related(Users).one(conn).await {
                Ok(Some(user)) => user,
                _ => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"message" : "帖子查询失败"})),
                    ));
                }
            };
            let media_list = match media::Entity::find()
                .filter(media::Column::PostId.eq(post.id))
                .order_by_asc(media::Column::Sort)
                .all(conn)
                .await
            {
                Ok(media_list) => media_list,
                Err(err) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "message": format!("查询帖子失败: {:?}", err) })),
                    ));
                }
            };
            let post_response = PostResponse {
                id: post.id,
                user_id: post.user_id,
                nickname: user.nickname,
                media_list,
                description: post.description,
                status: post.status,
                created_at: post.created_at,
                updated_at: post.updated_at,
            };
            Ok(Json(post_response))
        }
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"message":"未找到帖子"})))),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message" : "帖子查询失败"})),
        )),
    }
}

pub async fn search_post(
    State(state): State<Arc<AppState>>,
    query: Query<PostSearchParams>,
) -> Result<Json<PostSearchResults>, (StatusCode, Json<Value>)> {
    let keywords = &query.keywords;
    let meilisearch_client = &state.meilisearch_client;

    match meilisearch_client
        .index("posts")
        .search()
        .with_query(keywords)
        .execute::<PostSearchResult>()
        .await
    {
        Ok(posts) => {
            let mut results: Vec<PostSearchResult> = vec![];

            for result in posts.hits {
                results.push(result.result)
            }
            let result = PostSearchResults { hits: results };
            Ok(Json(result))
        }
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message" : "帖子查询失败"})),
        )),
    }
}
