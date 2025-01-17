//! Loading and saving data to sqlite database

use std::env;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use simplelog::warn;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::SqlitePool;
use tokio::sync::OnceCell;

pub mod config;
pub mod profiles;

static POOL: OnceCell<SqlitePool> = OnceCell::const_new();

fn get_pool() -> Result<&'static SqlitePool> {
    match POOL.get() {
        None => Err(anyhow!("Could not acquire Sqlite Pool")),
        Some(pool) => Ok(pool),
    }
}

pub async fn initialize_pool(database_url: Option<&str>) -> Result<()> {
    let database_url = if let Ok(database_url) = env::var("DATABASE_URL") {
        database_url
    } else if let Some(database_url) = database_url {
        database_url.to_string()
    } else {
        warn!("Environment variable `DATABASE_URL` not set and --db flag not provided. Using default URL.");
        String::from("sqlite:./data/hitomi.db")
    };

    let database_url = if database_url.contains("sqlite:") {
        database_url
    } else {
        format!("sqlite:{database_url}")
    };

    let options =
        SqliteConnectOptions::from_str(&database_url)?.journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePool::connect_with(options).await?;

    POOL.get_or_init(|| async { pool }).await;

    Ok(())
}
