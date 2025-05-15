use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Branch {
    pub id: i64,
    pub author_id: i64,
    pub original_timeline_id: i64,
    pub description: String,
    pub area_start: u64,
    pub area_end: u64,
}
