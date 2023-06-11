use axum::body::Bytes;
use axum::extract::multipart::{Field, MultipartError};
use axum::extract::Multipart;
use axum::http::{header, HeaderName};
use axum::response::{AppendHeaders, IntoResponse};
use axum::Json;
use reqwest::StatusCode;
use serde_json::{json, Value};

pub async fn upload_image(
    mut multipart: Multipart,
) -> Result<(AppendHeaders<[(HeaderName, String); 1]>, Bytes), (StatusCode, Json<Value>)> {
    println!("ewe");
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let content_type = field.content_type().unwrap().to_owned();
        let data = field.bytes().await.unwrap();
        // println!("Length of `{}` is {} bytes, content type is {}", name, data.len(), content_type);
        if name.eq("file") {
            println!("{} is upload", name);
            return Ok((AppendHeaders([(header::CONTENT_TYPE, content_type)]), data));
        }
    }
    Err((
        StatusCode::NOT_FOUND,
        Json(json!({"message":"ファイルがアップロードされていません"})),
    ))
}
