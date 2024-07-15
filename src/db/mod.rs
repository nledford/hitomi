//! Loading and saving data to sqlite database

use std::env;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::SqlitePool;
use tokio::sync::OnceCell;

pub static POOL: OnceCell<SqlitePool> = OnceCell::const_new();

pub async fn initialize_pool() -> Result<()> {
    match env::var("DATABASE_URL") {
        Ok(url) => {
            let options = SqliteConnectOptions::from_str(&url)?
                .journal_mode(SqliteJournalMode::Wal);

            let pool = SqlitePool::connect_with(options).await?;

            POOL.get_or_init(|| async { pool }).await;

            Ok(())
        }
        Err(_) => {
            Err(anyhow!("Environment variable `DATABASE_URL` is not set."))
        }
    }
}