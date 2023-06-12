// use std::fs::File;
// use std::future::Future;
// use std::io::Write;

use axum::body::Bytes;
use axum::extract::multipart::MultipartError;
use axum::extract::Multipart;
use axum::http::{header, HeaderName};
use axum::response::AppendHeaders;
use axum::Json;
use reqwest::StatusCode;
use sea_orm::Order::Field;
use serde_json::{json, Value};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

struct ImageFromUpload {
    length: usize,
    file_name: String,
    content_type: String,
    bytes: Bytes,
}

pub async fn upload_image(
    mut multipart: Multipart,
) -> Result<(AppendHeaders<[(HeaderName, String); 1]>, Bytes), (StatusCode, Json<Value>)> {
    while let Ok(field_option) = multipart.next_field().await {
        return if let Some(field) = field_option {
            let field_name = match field.name() {
                Some(field_name) => field_name,
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
                        Json(json!({"message":"file field required!"})),
                    ));
                }
            };
            let file_path = "assets/housr/wrer/fsdfsdfs/df/sdf/sfd/fs/fein.jpg";
            let mut file = File::create(file_path).await.unwrap();
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
