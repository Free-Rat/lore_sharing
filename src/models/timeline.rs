use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Timeline {
    pub id: i64,
    pub author_id: i64,
    pub description: String,
    pub start: u64,
    pub end: u64,
    pub unit: String, // e.g., "seconds", "minutes", "hours", "days", "weeks"
    pub universe_name: String,
}
