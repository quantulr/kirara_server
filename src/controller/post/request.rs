use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishPostRequest {
    pub description: Option<String>,
    pub media_ids: Vec<i32>,
}
