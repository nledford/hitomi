//! Loading and saving data to sqlite database

use std::env;
use std::str::FromStr;

use anyhow::Result;
use simplelog::warn;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::SqlitePool;
use tokio::sync::OnceCell;

pub mod config;
pub mod profiles;

pub static POOL: OnceCell<SqlitePool> = OnceCell::const_new();

pub async fn initialize_pool() -> Result<()> {
    let database_url = if let Ok(database_url) = env::var("DATABASE_URL") {
        database_url
    } else {
        warn!("Environment variable `DATABASE_URL` not set. Using default URL.");
        String::from("sqlite:./data/hitomi.db")
    };

    let options =
        SqliteConnectOptions::from_str(&database_url)?.journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePool::connect_with(options).await?;

    POOL.get_or_init(|| async { pool }).await;

    Ok(())
}
