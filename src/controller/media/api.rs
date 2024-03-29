use std::path::PathBuf;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Multipart, Path, State};
use axum::http::{header, HeaderValue, Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum_extra::{
    headers::authorization::{Authorization, Bearer},
    TypedHeader,
};

use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{json, Value};
use tokio::io::AsyncWriteExt;
use tower::ServiceExt;
use tower_http::services::ServeFile;

use crate::entities::media;
use crate::utils::dir::create_dir;
use crate::utils::media::{get_video_thumbnail, is_image, is_media, is_video};
use crate::utils::user::get_user_from_token;
use crate::AppState;

pub async fn upload_media(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    mut multipart: Multipart,
) -> Result<Json<media::Model>, (StatusCode, Json<Value>)> {
    let upload_path = &state.upload_path; // 上传路径
    let conn = &state.conn; // 数据库连接
    let token = auth.token(); // token

    let uid = match get_user_from_token(token, &state.jwt_secret, conn).await {
        Some(user) => user.id,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "message": "用户未登录！"
                })),
            ));
        }
    };

    while let Ok(Some(field)) = multipart.next_field().await {
        match field
            .name()
            .and_then(|name| if name.eq("file") { Some(name) } else { None })
        {
            None => {
                continue;
            }
            _ => {}
        }

        let (file_name, content_type) = match (field.file_name(), field.content_type()) {
            (Some(file_name), Some(content_type)) => {
                if !is_media(&content_type) {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "message": "不支持的文件类型！"
                        })),
                    ));
                }
                (file_name.to_owned(), content_type.to_owned())
            }
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "message": "上传内容为空！"
                    })),
                ));
            }
        };

        // 获取文件后缀名
        let file_ext = match PathBuf::from(&file_name)
            .extension()
            .and_then(|ext| ext.to_str())
        {
            Some(ext) => ext.to_owned(),
            None => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "message": "文件名不合法！"
                    })),
                ));
            }
        };

        let rand_uuid = uuid::Uuid::new_v4().to_string().replace("-", "");
        // 生成存储文件名
        let store_file_name = format!("{}.{}", &rand_uuid, file_ext);

        let datetime_utc_now = chrono::Utc::now();
        // 生成相对路径
        let relative_path = datetime_utc_now.format("%Y/%m/%d").to_string();
        // 生成存储文件路径
        let file_store_path = std::path::Path::new(upload_path)
            .join(&relative_path)
            .join(&store_file_name);

        let file_bytes = match field.bytes().await {
            Ok(bytes) => bytes,
            Err(err) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "message": format!("读取文件失败！{}", err)
                    })),
                ));
            }
        };

        match create_dir(format!("{}/{}", upload_path, &relative_path).as_str()).await {
            Err(err) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "message": format!("创建目录失败！{}", err)
                    })),
                ));
            }
            Ok(_) => {}
        };
        let mut file = match tokio::fs::File::create(&file_store_path).await {
            Ok(file) => file,
            Err(err) => {
                println!("err: {}", err);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "message": format!("存储文件失败！{}", err)
                    })),
                ));
            }
        };
        return match file.write_all(&file_bytes).await {
            Ok(_) => {
                if is_video(&content_type) {
                    let _thumb_path = get_video_thumbnail(&file_store_path.to_str().unwrap()).await;
                }

                let resp = media::ActiveModel {
                    user_id: Set(uid),
                    name: Set(file_name),
                    path: Set(format!("{}/{}", &relative_path, &store_file_name)),
                    mime_type: Set(content_type),
                    size: Set(file_bytes.len() as u64),
                    ..Default::default()
                }
                .insert(conn)
                .await;
                match resp {
                    Ok(res) => Ok(Json(res)),
                    Err(err) => Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "message": format!("存储文件失败！{}", err)
                        })),
                    )),
                }
            }
            Err(err) => {
                println!("err: {}", err);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "message": format!("存储文件失败！{}", err)
                    })),
                ))
            }
        };
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(json!({
            "message": "上传内容为空！"
        })),
    ))
}

pub async fn get_media_trunk(
    State(state): State<Arc<AppState>>,
    Path((year, month, day, file_name)): Path<(String, String, String, String)>,
    req: Request<Body>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let relative_path = format!("{}/{}/{}/{}", year, month, day, file_name);
    let upload_path = &state.upload_path;
    let store_path = std::path::Path::new(upload_path).join(&relative_path);
    let media_model = match media::Entity::find()
        .filter(media::Column::Path.eq(&relative_path))
        .one(&state.conn)
        .await
    {
        Ok(Some(media)) => media,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "message": "文件不存在！"
                })),
            ));
        }
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": format!("获取文件失败！{}", err)
                })),
            ));
        }
    };
    match ServeFile::new(&store_path).oneshot(req).await {
        Ok(mut res) => {
            res.headers_mut().insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(format!("filename={}", media_model.name).as_str()).unwrap(),
            );
            Ok(res)
        }
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": format!("获取文件失败！{}", err)
            })),
        )),
    }
}

pub async fn get_media_thumb(
    State(state): State<Arc<AppState>>,
    Path((year, month, day, file_name)): Path<(String, String, String, String)>,
    req: Request<Body>,
) -> Result<Response, (StatusCode, Json<Value>)> {
    let conn = &state.conn;
    let upload_path = &state.upload_path;
    let wsrv_nl_port = &state.wsrv_nl_port;
    let media = match media::Entity::find()
        .filter(media::Column::Path.eq(format!("{}/{}/{}/{}", year, month, day, file_name)))
        .one(conn)
        .await
    {
        Ok(Some(media)) => media,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "message": "文件不存在！"
                })),
            ));
        }
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": format!("获取文件失败！{}", err)
                })),
            ));
        }
    };
    let content_type = &media.mime_type;
    let video_thumbnail_store_path =
        std::path::Path::new(upload_path).join(format!("{}.thumbnail.jpg", &media.path));

    if is_image(content_type) {
        let response = match reqwest::get(format!(
            "http://localhost:{}/?url=http://172.17.0.1:3000/v/s/{}&w=512",
            wsrv_nl_port, &media.path
        ))
        .await
        {
            Ok(res) => res,
            Err(err) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "message": format!("获取文件失败！{}", err)
                    })),
                ));
            }
        };
        let resp_builder = Response::builder();
        Ok(resp_builder
            .body(Body::from_stream(response.bytes_stream()))
            .unwrap())
    } else if is_video(content_type) {
        match ServeFile::new(&video_thumbnail_store_path)
            .oneshot(req)
            .await
        {
            Ok(mut res) => {
                res.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str("image/jpeg").unwrap(),
                );
                Ok(res.into_response())
            }
            Err(err) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": format!("获取文件失败！{}", err)
                })),
            )),
        }
    } else {
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "文件类型不支持！"
            })),
        ))
    }
}
