use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Universe {
    // The unique identifier; using name as the primary key
    pub name: String,
    pub description: String,
}
