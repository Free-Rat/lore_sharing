use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TimelineMerge {
    pub id: i64,
    pub source_timeline_id: i64,
    pub target_timeline_id: i64,
    pub merged_at: String,
}
