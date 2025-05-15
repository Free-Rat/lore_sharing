use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Event {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub reference: String,
    pub image: Option<String>,     // URL or path
    pub thumbnail: Option<String>, // URL or path
}
