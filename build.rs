// build.rs
use std::{fs::File, path::Path};

fn main() {
    dotenv::dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for build");
    let path = url
        .strip_prefix("sqlite://")
        .expect("DATABASE_URL must start with sqlite://");
    let db_path = Path::new(path);
    if let Some(dir) = db_path.parent() {
        std::fs::create_dir_all(dir).unwrap();
    }
    if !db_path.exists() {
        File::create(db_path).unwrap();
    }
}
