use anyhow::Result;

use crate::db::models::{DbProfile, DbProfileSection};
use crate::db::POOL;

pub async fn get_profiles() -> Result<Vec<DbProfile>> {
    let profiles = sqlx::query_as::<_, DbProfile>("select * from profile")
        .fetch_all(POOL.get().unwrap())
        .await?;

    Ok(profiles)
}

pub async fn get_profile_sections(profile_id: i32) -> Result<Vec<DbProfileSection>> {
    let sections = sqlx::query_as::<_, DbProfileSection>("select * from profile_section where profile_id = ?")
        .bind(profile_id)
        .fetch_all(POOL.get().unwrap())
        .await?;

    Ok(sections)
}