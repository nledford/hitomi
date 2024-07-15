use anyhow::Result;
use sqlx::{Encode, Sqlite};

use crate::config::{Config as AppConfig, ConfigBuilder};
use crate::db::POOL;

#[derive(sqlx::FromRow)]
struct DbConfig {
    name: String,
    value: String,
}

pub async fn have_config() -> Result<bool> {
    let result: Option<(i32,)> = sqlx::query_as("select count(*) from config")
        .fetch_optional(POOL.get().unwrap())
        .await?;

    let result = if let Some(result) = result {
        result.0 > 0
    } else {
        false
    };

    Ok(result)
}

async fn add_config_setting<'q, T: 'q + Send + Encode<'q, Sqlite> + sqlx::Type<Sqlite>>(
    name: &'q str,
    value: T,
) -> Result<()> {
    sqlx::query(
        r#"
        insert into config
        values (?, ?)
    "#,
    )
        .bind(name)
        .bind(value)
        .execute(POOL.get().unwrap())
        .await?;

    Ok(())
}

pub async fn save_config(config: &AppConfig) -> Result<()> {
    add_config_setting("plex_token", config.get_plex_token()).await?;
    add_config_setting("plex_url", config.get_plex_url()).await?;
    add_config_setting("primary_section_id", config.get_primary_section_id()).await?;

    Ok(())
}

pub async fn fetch_config() -> Result<AppConfig> {
    let rows = sqlx::query_as::<_, DbConfig>(
        r#"
        select * from config
    "#,
    )
        .fetch_all(POOL.get().unwrap())
        .await?;

    let mut config = ConfigBuilder::default();
    for row in rows {
        if row.name == "plex_token" {
            config.plex_token(row.value);
            continue;
        }

        if row.name == "plex_url" {
            config.plex_url(row.value);
            continue;
        }

        if row.name == "primary_section_id" {
            config.primary_section_id(row.value.parse().unwrap());
            continue;
        }
    }

    Ok(config.build().unwrap())
}
