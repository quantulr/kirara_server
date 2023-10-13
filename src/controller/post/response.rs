use crate::entities::media;
use sea_orm::prelude::DateTimeUtc;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostListResponse {
    pub total: u64,
    pub items: Vec<PostResponse>,
    pub prev: Option<i32>,
    pub next: Option<i32>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostResponse {
    pub id: i32,
    pub user_id: i32,
    pub media_list: Vec<media::Model>,
    pub description: Option<String>,
    pub status: i8,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
