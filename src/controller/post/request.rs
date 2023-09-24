use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishPostRequest {
    pub description: Option<String>,
    pub media_ids: Vec<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    pub before: Option<i64>,
    pub after: Option<i64>,
    pub per_page: u64,
}
