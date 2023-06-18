use crate::entities::images;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageHistoryResponse {
    pub total: u64,
    pub total_pages: u64,
    pub has_next: bool,
    pub items: Vec<images::Model>,
}
