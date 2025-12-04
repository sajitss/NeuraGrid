use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::fs::File;
use std::path::Path;

pub async fn init_db() -> Result<Pool<Sqlite>, sqlx::Error> {
    let db_path = "neuragrid.db";
    
    if !Path::new(db_path).exists() {
        println!("Creating database file: {}", db_path);
        File::create(db_path).expect("Failed to create database file");
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&format!("sqlite://{}", db_path))
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS jobs (
            id TEXT PRIMARY KEY,
            job_type TEXT NOT NULL,
            args TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
