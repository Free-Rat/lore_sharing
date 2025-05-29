use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TimelineEvent {
    pub timeline_id: i64,
    pub event_id: i64,
    pub position: i64,
}
