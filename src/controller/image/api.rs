// use std::fs::File;
// use std::future::Future;
// use std::io::Write;

use std::path::Path;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Multipart, State};
use axum::extract::multipart::MultipartError;
use axum::http::{header, HeaderName};
use axum::Json;
use axum::response::AppendHeaders;
use image::ImageResult;
use reqwest::StatusCode;
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::Set;
use sea_orm::Order::Field;
use serde_json::{json, Value};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{AppState, create_dir};
use crate::entities::images;

struct ImageFromUpload {
    length: usize,
    file_name: String,
    content_type: String,
    bytes: Bytes,
}

// pub async fn upload_test(mut multipart: Multipart) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
//     while let Ok(field_option) = multipart.next_field().await {
//         if let Some(field) = field_option {
//             let file_name = &field.file_name();
//             let content_type = &field.content_type();
//             let bytes = &field.bytes().await;
//             match (file_name, content_type, bytes) {
//                 (Some(file_name), Some(content_type), Ok(bytes)) => {}
//                 _ => {}
//             }
//         } else {};
//     }
//     Ok(Json(json!({"fdsaf":"fesfsdf"})))
// }

pub async fn upload_image(State(state): State<Arc<AppState>>,
                          mut multipart: Multipart,
) -> Result<Json<images::Model>, (StatusCode, Json<Value>)> {
    let upload_path = &state.upload_path;
    while let Ok(field_option) = multipart.next_field().await {
        return if let Some(field) = field_option
        /*.and_then(|field| {
            if let Some(field_name) = field.name() {
                if field_name.eq("file") {
                    Some(field)
                } else {
                    None
                }
            } else {
                None;
            }
        })*/
        {
            let field_name = match field.name() {
                //
                Some(field_name) => {
                    if field_name.eq("file") {
                        field_name
                    } else {
                        continue;
                    }
                }
                None => {
                    return Err((
                        StatusCode::NOT_FOUND,
                        Json(json!({"message":"file field required!"})),
                    ));
                }
            };
            let file_name = match field.file_name() {
                Some(file_name) => file_name.to_owned(),
                None => {
                    return Err((
                        StatusCode::NOT_FOUND,
                        Json(json!({"message":"file field required!"})),
                    ));
                }
            };
            let content_type = match field.content_type() {
                Some(content_type) => content_type.to_owned(),
                None => {
                    return Err((
                        StatusCode::NOT_FOUND,
                        Json(json!({"message":"file field required!"})),
                    ));
                }
            };

            let file_bytes = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(_) => {
                    return Err((
                        StatusCode::NOT_FOUND,
                        Json(json!({"message":"file upload failed!"})),
                    ));
                }
            };
            let extensions = match mime_guess::get_mime_extensions_str(content_type.as_str()) {
                Some(ext) => ext.first().unwrap(),
                None => {
                    return Err((
                        StatusCode::NOT_FOUND,
                        Json(json!({"message":"unknown file type!"})),
                    ));
                }
            };
            let image_data = image::load_from_memory(&file_bytes);
            struct Size {
                width: u32,
                height: u32,
            }
            let size = match image_data {
                Ok(image) => {
                    Size {
                        width: image.width(),
                        height: image.height(),
                    }
                }
                Err(_) => {
                    return Err((
                        StatusCode::NOT_FOUND,
                        Json(json!({"message":"unknown file type!"})),
                    ));
                }
            };

            let datetime_now = chrono::Local::now();
            let formatted_date = datetime_now.format("%Y/%m/%d").to_string();
            let timestamp_now = datetime_now.timestamp_millis();
            let target_file_name = format!("{}.{}", Uuid::new_v4().to_string().replace("-", ""), extensions);
            let target_directory = Path::new(upload_path).join(&formatted_date);

            create_dir(target_directory.to_str().unwrap()).await.expect("unable save file");

            let target_file_path = target_directory.join(&target_file_name);
            let mut file = File::create(target_file_path).await.expect("unable save file");
            let res = file.write_all(file_bytes.as_ref()).await;

            match res {
                Ok(_) => {
                    let conn = &state.conn;
                    let model = images::ActiveModel {
                        file_path: Set(format!("{}/{}", &formatted_date, &target_file_name).to_owned()),
                        file_name: Set(file_name.to_owned()),
                        size: Set(file_bytes.len() as u64),
                        width: Set(size.width as u64),
                        height: Set(size.height as u64),
                        upload_time: Set(timestamp_now as u64),
                        uid: Set(1),
                        ..Default::default()
                    }.insert(conn).await.expect("failed");
                    Ok(Json(model))
                }
                Err(err) => {
                    Err((
                        StatusCode::NOT_FOUND,
                        Json(json!({"message":"file field required!"})),
                    ))
                }
            }
        } else {
            Err((
                StatusCode::NOT_FOUND,
                Json(json!({"message":"file field required!"})),
            ))
        };
    }
    Err((
        StatusCode::NOT_FOUND,
        Json(json!({"message":"ファイルがアップロードされていません"})),
    ))
}
