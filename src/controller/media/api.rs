use std::path::PathBuf;
use std::sync::Arc;

use axum::{Json, TypedHeader};
use axum::body::StreamBody;
use axum::extract::{Multipart, Path, State};
use axum::headers::{Authorization, HeaderName};
use axum::headers::authorization::Bearer;
use axum::http::{header, StatusCode};
use axum::response::AppendHeaders;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use sea_orm::ActiveValue::Set;
use serde_json::{json, Value};
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;
use tower_http::services::ServeFile;

use crate::AppState;
use crate::entities::media;
use crate::utils::dir::create_dir;
use crate::utils::media_type::is_media;
use crate::utils::user::get_user_from_token;

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
        // 生成存储文件名
        let store_file_name = format!("{}.{}", uuid::Uuid::new_v4().to_string().replace("-", ""), file_ext);

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
                let resp = media::ActiveModel {
                    user_id: Set(uid),
                    name: Set(file_name),
                    path: Set(format!("{}/{}", &relative_path, &store_file_name)),
                    mime_type: Set(content_type),
                    size: Set(file_bytes.len() as i32),
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

pub async fn get_media(
    State(state): State<Arc<AppState>>,
    Path((year, month, day, file_name)): Path<(String, String, String, String)>,
) -> Result<
    (
        AppendHeaders<[(HeaderName, String); 2]>,
        StreamBody<ReaderStream<tokio::fs::File>>,
    ),
    (StatusCode, Json<Value>),
> {
    let res_file = ServeFile::new("/Volumes/iMac Doc/Pictures/upload/2023/06/30/910a081669fc458c9fae01f3ba88b351.mp4");
    let conn = &state.conn;
    let path = format!("{}/{}/{}/{}", year, month, day, file_name); // 生成文件路径
    let media = match media::Entity::find()
        .filter(media::Column::Path.eq(&path))
        .one(conn).await {
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
    let file_path = format!("{}/{}", &state.upload_path, &path);
    let body = match tokio::fs::File::open(&file_path).await {
        Ok(file) => {
            let stream = ReaderStream::new(file);
            StreamBody::from(stream)
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
    Ok((
        AppendHeaders([
            (header::CONTENT_TYPE, media.mime_type),
            (
                header::CONTENT_DISPOSITION,
                format!("filename=\"{}\"", media.name),
            ),
        ]),
        body,
    ))
}
