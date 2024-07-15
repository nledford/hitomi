#![allow(dead_code)]

use anyhow::Result;
use sqlx::Row;

use crate::db;
use crate::db::models::{DbProfile, DbProfileSection};
use crate::db::POOL;
use crate::profiles::profile::Profile;
use crate::profiles::profile_section::ProfileSection;
use crate::profiles::SectionType;

pub async fn create_profile(new_profile: &Profile) -> Result<()> {
    let result = sqlx::query(
        r#"
        insert into profile (playlist_id,
                     profile_title,
                     profile_summary,
                     enabled,
                     profile_source,
                     profile_source_id,
                     refresh_interval,
                     time_limit,
                     track_limit)
        values (?,?,?,?,?,?,?,?,?)
        returning id
    "#,
    )
        .bind(new_profile.get_playlist_id().as_str())
        .bind(new_profile.get_title())
        .bind(new_profile.get_summary())
        .bind(true) // enabled
        .bind(new_profile.get_profile_source().to_string())
        .bind(new_profile.get_profile_source_id_str())
        .bind(new_profile.get_refresh_interval())
        .bind(new_profile.get_time_limit())
        .bind(new_profile.get_track_limit())
        .fetch_one(POOL.get().unwrap())
        .await?;

    let profile_id = result.get(0);

    for section in new_profile.get_sections() {
        create_profile_section(profile_id, section).await?;
    }

    Ok(())
}

async fn create_profile_section(profile_id: i32, section: &ProfileSection) -> Result<()> {
    sqlx::query(
        r#"
        insert into profile_section (profile_id,
                             section_type,
                             enabled,
                             deduplicate_tracks_by_guid,
                             deduplicate_tracks_by_title_and_artist,
                             maximum_tracks_by_artist,
                             minimum_track_rating,
                             randomize_tracks,
                             sorting)
        VALUES(?,?,?,?,?,?,?,?,?)
    "#,
    )
        .bind(profile_id)
        .bind(section.get_section_type())
        .bind(true) // enabled
        .bind(section.get_deduplicate_tracks_by_guid())
        .bind(section.get_deduplicate_tracks_by_title_and_artist())
        .bind(section.get_maximum_tracks_by_artist())
        .bind(section.get_minimum_track_rating())
        .bind(section.get_randomize_tracks())
        .bind(section.get_sorting())
        .execute(POOL.get().unwrap())
        .await?;

    Ok(())
}

async fn fetch_profile(profile_id: i32) -> Result<DbProfile> {
    let profile = sqlx::query_as::<_, DbProfile>("select * from profile where profile_id = ?")
        .bind(profile_id)
        .fetch_one(POOL.get().unwrap())
        .await?;

    Ok(profile)
}

async fn fetch_profile_id(profile_title: &str) -> Result<Option<i32>> {
    let row: Option<(i32,)> =
        sqlx::query_as("select profile_id from profile where profile_title = ?")
            .bind(profile_title)
            .fetch_optional(POOL.get().unwrap())
            .await?;

    let id = row.map(|row| row.0);

    Ok(id)
}

async fn delete_profile(profile_id: i32) -> Result<()> {
    sqlx::query("delete from profile where profile_id = ?")
        .bind(profile_id)
        .execute(POOL.get().unwrap())
        .await?;

    Ok(())
}

async fn update_profile(profile: &Profile) -> Result<()> {
    let profile_id = fetch_profile_id(profile.get_title()).await?.unwrap();

    sqlx::query(
        r#"
        update profile
        set profile_title = ?,
            profile_summary = ?,
            enabled = ?,
            profile_source = ?,
            profile_source_id = ?,
            refresh_interval = ?,
            time_limit = ?,
            track_limit = ?
        where profile_id = ?
    "#,
    )
        .bind(profile.get_title())
        .bind(profile.get_summary())
        .bind(profile.get_enabled())
        .bind(profile.get_profile_source().to_string())
        .bind(profile.get_profile_source_id_str())
        .bind(profile.get_refresh_interval())
        .bind(profile.get_time_limit())
        .bind(profile.get_track_limit())
        .bind(profile_id)
        .execute(POOL.get().unwrap())
        .await?;

    for section in profile.get_sections() {
        update_profile_section(profile_id, section).await?;
    }

    Ok(())
}

async fn update_profile_section(profile_id: i32, section: &ProfileSection) -> Result<()> {
    let profile_section_id = fetch_profile_section_id(profile_id, section.get_section_type())
        .await?
        .unwrap();

    sqlx::query(
        r#"
        update profile_section
        set enabled = ?,
           deduplicate_tracks_by_guid = ?,
           deduplicate_tracks_by_title_and_artist = ?,
           maximum_tracks_by_artist = ?,
           minimum_track_rating = ?,
           randomize_tracks = ?,
           sorting = ?
        where profile_id = ? and profile_section_id = ?
    "#,
    )
        .bind(section.is_enabled())
        .bind(section.get_deduplicate_tracks_by_guid())
        .bind(section.get_deduplicate_tracks_by_title_and_artist())
        .bind(section.get_maximum_tracks_by_artist())
        .bind(section.get_minimum_track_rating())
        .bind(section.get_randomize_tracks())
        .bind(section.get_sorting())
        .bind(profile_id)
        .bind(profile_section_id)
        .execute(POOL.get().unwrap())
        .await?;

    Ok(())
}

async fn fetch_profile_section_id(
    profile_id: i32,
    section_type: SectionType,
) -> Result<Option<i32>> {
    let row: Option<(i32,)> = sqlx::query_as(
        r#"
        select profile_section_id
        from profile_section
        where profile_id = ? and section_type = ?
    "#,
    )
        .bind(profile_id)
        .bind(section_type)
        .fetch_optional(POOL.get().unwrap())
        .await?;

    let id = row.map(|row| row.0);

    Ok(id)
}

async fn fetch_profiles() -> Result<Vec<DbProfile>> {
    let profiles = sqlx::query_as::<_, DbProfile>(
        r#"
        select *
        from profile
        order by profile_title
    "#,
    )
        .fetch_all(POOL.get().unwrap())
        .await?;

    Ok(profiles)
}

async fn fetch_profile_sections(profile_id: i32) -> Result<Vec<DbProfileSection>> {
    let sections =
        sqlx::query_as::<_, DbProfileSection>("select * from profile_section where profile_id = ?")
            .bind(profile_id)
            .fetch_all(POOL.get().unwrap())
            .await?;

    Ok(sections)
}

pub async fn fetch_all_data() -> Result<Vec<Profile>> {
    let db_profiles = fetch_profiles().await?;
    let mut profiles = vec![];

    for db_profile in db_profiles {
        let db_sections = fetch_profile_sections(db_profile.profile_id).await?;
        let profile = db::db_profile_to_profile(db_profile, db_sections);
        profiles.push(profile)
    }

    Ok(profiles)
}
