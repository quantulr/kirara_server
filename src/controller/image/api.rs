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
use reqwest::StatusCode;
use sea_orm::Order::Field;
use serde_json::{json, Value};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{AppState, create_dir};

struct ImageFromUpload {
    length: usize,
    file_name: String,
    content_type: String,
    bytes: Bytes,
}

pub async fn upload_image(State(state): State<Arc<AppState>>,
                          mut multipart: Multipart,
) -> Result<(AppendHeaders<[(HeaderName, String); 1]>, Bytes), (StatusCode, Json<Value>)> {
    let upload_path = &state.upload_path;
    while let Ok(field_option) = multipart.next_field().await {
        return if let Some(field) = field_option {
            let field_name = match field.name() {
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

            let formatted_date = chrono::Local::now().format("%Y/%m/%d").to_string();
            let target_file_name = format!("{}.{}", Uuid::new_v4().to_string().replace("-", ""), extensions);
            let target_directory = Path::new(upload_path).join(formatted_date);
            create_dir(target_directory.to_str().unwrap()).await.expect("unable save file");
            let target_file_path = target_directory.join(target_file_name);
            let mut file = File::create(target_file_path).await.unwrap();
            let res = file.write_all(file_bytes.as_ref()).await;

            Ok((
                AppendHeaders([(header::CONTENT_TYPE, content_type)]),
                file_bytes,
            ))
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
