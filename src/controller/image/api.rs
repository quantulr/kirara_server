use std::path::PathBuf;
use std::sync::Arc;

use axum::body::StreamBody;
use axum::extract::{Multipart, Path, Query, State, TypedHeader};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::{header, HeaderName};
use axum::response::{AppendHeaders, IntoResponse};
use axum::Json;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use reqwest::StatusCode;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use serde_json::{json, Value};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::controller::image::request::Pagination;
use crate::controller::image::response::ImageHistoryResponse;
use crate::controller::user::response::Claims;
use crate::entities::{images, users};
use crate::utils::dir::create_dir;
use crate::utils::media_type::{get_content_type, is_image};
use crate::AppState;

// 上传图片
pub async fn upload_image(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    mut multipart: Multipart,
) -> Result<Json<images::Model>, (StatusCode, Json<Value>)> {
    let upload_path = &state.upload_path; // 上传路径
    let conn = &state.conn; // 数据库连接
    let token = auth.token(); // token

    // 从token中获取用户名
    let username = match jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(&state.jwt_secret.as_ref()),
        &Validation::new(Algorithm::HS512),
    ) {
        Ok(data) => data.claims.username,
        Err(err) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "message": format!("文件上传失败！{}", err)
                })),
            ));
        }
    };
    // 从数据库中获取用户id
    let uid = match users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .one(conn)
        .await
    {
        Ok(Some(user)) => user.id,
        Err(err) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "message": format!("文件上传失败！{}", err)
                })),
            ));
        }
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({"message":"文件上传失败！"})),
            ));
        }
    };

    while let Ok(Some(field)) = multipart.next_field().await {
        // 获取字段名，如果不是file，则返回错误
        match field.name() {
            Some(field_name) => {
                if field_name.eq("file") {
                    field_name
                } else {
                    return Err((
                        StatusCode::NOT_FOUND,
                        Json(json!({"message":"文件上传失败！"})),
                    ));
                }
            }
            None => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"message":"文件上传失败！dsdsdsd"})),
                ));
            }
        };

        // 获取content_type
        let content_type = match field.content_type() {
            Some(content_type) => content_type.to_owned(),
            None => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"message":"文件上传失败！"})),
                ));
            }
        };

        // 判断是否是图片
        if !is_image(content_type.as_str()) {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({"message":"请上传图片！"})),
            ));
        }

        // 获取文件名
        let file_name = match field.file_name() {
            Some(file_name) => file_name.to_owned(),
            None => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"message":"文件上传失败！"})),
                ));
            }
        };

        // 获取文件后缀名
        let file_extension = match PathBuf::from(&file_name).extension() {
            Some(file_extension) => file_extension.to_str().unwrap().to_owned(),
            None => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"message":"文件上传失败！"})),
                ));
            }
        };

        // 读取文件内容
        let file_bytes = match field.bytes().await {
            Ok(bytes) => bytes,
            Err(_) => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"message":"文件上传失败！"})),
                ));
            }
        };
        let image_data = image::load_from_memory(&file_bytes);

        // 获取图片尺寸
        struct Size {
            width: u32,
            height: u32,
        }
        let size = match image_data {
            Ok(image) => Size {
                width: image.width(),
                height: image.height(),
            },
            Err(_) => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"message":"unknown file type!443434"})),
                ));
            }
        };

        let datetime_now = chrono::Local::now(); // 获取当前时间
        let formatted_date = datetime_now.format("%Y/%m/%d").to_string(); // 格式化时间
        let timestamp_now = datetime_now.timestamp_millis(); // 获取时间戳
        let target_file_name = format!(
            "{}.{}",
            Uuid::new_v4().to_string().replace("-", ""),
            file_extension
        ); // 生成文件名
        let target_directory = std::path::Path::new(upload_path).join(&formatted_date); // 生成目录

        create_dir(target_directory.to_str().unwrap())
            .await
            .expect("unable save file"); // 创建目录

        let target_file_path = target_directory.join(&target_file_name); // 生成文件路径
        let mut file = File::create(&target_file_path)
            .await
            .expect("unable save file"); // 创建文件
        let res = &file.write_all(file_bytes.as_ref()).await; // 写入文件

        // 保存到数据库
        return match res {
            Ok(_) => {
                let model_res = images::ActiveModel {
                    file_path: Set(format!("{}/{}", &formatted_date, &target_file_name).to_owned()),
                    file_name: Set(file_name.to_owned()),
                    size: Set(file_bytes.len() as u64),
                    width: Set(size.width as u64),
                    height: Set(size.height as u64),
                    upload_time: Set(timestamp_now as u64),
                    uid: Set(uid),
                    ..Default::default()
                }
                .insert(conn)
                .await;
                let model = match model_res {
                    Ok(model) => model,
                    Err(db_err) => {
                        let err_msg = db_err.to_string(); // 获取错误信息
                                                          // 从本地删除文件
                        let _ = tokio::fs::remove_file(&target_file_path).await;
                        return Err((StatusCode::NOT_FOUND, Json(json!({ "message": &err_msg }))));
                    }
                };
                Ok(Json(model))
            }
            Err(_err) => Err((
                StatusCode::NOT_FOUND,
                Json(json!({"message":"文件上传失败！"})),
            )),
        };
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(json!({"message":"上传内容为空！"})),
    ))
}

// 获取图片
pub async fn get_image(
    State(state): State<Arc<AppState>>,
    Path((year, month, day, file_name)): Path<(String, String, String, String)>,
) -> Result<
    (
        AppendHeaders<[(HeaderName, String); 2]>,
        StreamBody<ReaderStream<tokio::fs::File>>,
    ),
    (StatusCode, Json<Value>),
> {
    let conn = &state.conn;

    let path = format!("{}/{}/{}/{}", year, month, day, file_name); // 生成文件路径

    let content_type = match get_content_type(file_name.as_str()) {
        Some(content_type) => content_type,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({"message":"获取文件类型失败！"})),
            ));
        }
    }; // 获取文件类型

    let image = match images::Entity::find()
        .filter(images::Column::FilePath.eq(&path))
        .one(conn)
        .await
    {
        Ok(Some(image)) => image,
        _ => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({"message":"file not found!"})),
            ));
        }
    };
    let upload_path = &state.upload_path; // 获取上传路径
    let file_path = std::path::Path::new(upload_path).join(path.as_str()); // 生成文件路径

    // 判断文件是否存在
    let file = match File::open(file_path).await {
        Ok(file) => file,
        Err(_) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({"message":"file not found!"})),
            ));
        }
    };
    let stream = ReaderStream::new(file); // 生成流
    let body = StreamBody::new(stream); // 生成body

    Ok((
        AppendHeaders([
            (header::CONTENT_TYPE, content_type),
            (
                header::CONTENT_DISPOSITION,
                format!("filename={}", image.file_name),
            ),
        ]),
        body,
    ))
}

// 获取图片上传历史
pub async fn get_image_history(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    query: Query<Pagination>,
) -> Result<Json<ImageHistoryResponse>, (StatusCode, Json<Value>)> {
    let conn = &state.conn;
    let token = auth.token(); // token

    // 从token中获取用户名
    let username = match jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(&state.jwt_secret.as_ref()),
        &Validation::new(Algorithm::HS512),
    ) {
        Ok(data) => data.claims.username,
        Err(err) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({ "message": format!("图片获取失败, {}", err) })),
            ));
        }
    };
    // 从数据库中获取用户id
    let uid = match users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .one(conn)
        .await
    {
        Ok(Some(user)) => user.id,
        Err(err) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "message": format!("用户信息查询失败{}", err)
                })),
            ));
        }
        Ok(None) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"message":  "用户信息查询失败"})),
            ));
        }
    };

    let pagination = query.0;
    let page = pagination.page;
    let per_page = pagination.per_page;
    let paginator = images::Entity::find()
        .filter(images::Column::Uid.eq(uid))
        .paginate(conn, per_page);
    let images = paginator.fetch_page(page - 1).await;
    let items_and_pages_number = paginator.num_items_and_pages().await;
    return match (images, items_and_pages_number) {
        (Ok(images), Ok(items_and_pages_number)) => {
            let has_next = page < items_and_pages_number.number_of_pages;
            let image_history_response = ImageHistoryResponse {
                items: images,
                total: items_and_pages_number.number_of_items,
                total_pages: items_and_pages_number.number_of_pages,
                has_next,
            };
            Ok(Json(image_history_response))
        }
        _ => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"message":"图片查询失败！"})),
        )),
    };
}

pub async fn image_thumbnail(
    State(state): State<Arc<AppState>>,
    Path((year, month, day, file_name)): Path<(String, String, String, String)>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let path = format!("{}/{}/{}/{}", year, month, day, file_name); // 生成文件路径

    let body = match reqwest::get(format!(
        "https://wsrv.nl/?url=https://kirara.hodokencho.com/api/image/{}",
        path
    ))
    .await
    {
        Ok(res) => res.bytes_stream(),
        Err(err) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({"message":"获取缩略图失败！"})),
            ));
        }
    };
    let stream = StreamBody::new(body); // 生成流
    let mime = match get_content_type(file_name.as_str()) {
        Some(content_type) => content_type,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({"message":"获取文件类型失败"})),
            ));
        }
    };
    Ok((AppendHeaders([(header::CONTENT_TYPE, mime)]), stream))
}
