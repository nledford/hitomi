use std::str::FromStr;

use anyhow::Result;
use simplelog::debug;
use sqlx::Row;

use crate::db::POOL;
use crate::plex::types::PlexId;
use crate::profiles::profile::{Profile, ProfileBuilder};
use crate::profiles::profile_section::ProfileSection;
use crate::profiles::{ProfileSource, SectionType};
use crate::types::profiles::profile_source_id::ProfileSourceId;
use crate::types::profiles::refresh_interval::RefreshInterval;
use crate::types::Title;

// CREATE #####################################################################

pub async fn create_profile(
    playlist_id: &str,
    new_profile: &Profile,
    sections: &[ProfileSection],
) -> Result<()> {
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
        returning profile_id
    "#,
    )
    .bind(playlist_id)
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

    for section in sections {
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

// DELETE #####################################################################

pub async fn delete_profile(profile_id: i32) -> Result<()> {
    sqlx::query("delete from profile where profile_id = ?")
        .bind(profile_id)
        .execute(POOL.get().unwrap())
        .await?;

    Ok(())
}

// UPDATE #####################################################################

pub async fn update_profile(profile: &Profile, sections: &[ProfileSection]) -> Result<()> {
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

    for section in sections {
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
    .bind(section.get_minimum_track_rating_adjusted())
    .bind(section.get_randomize_tracks())
    .bind(section.get_sorting())
    .bind(profile_id)
    .bind(profile_section_id)
    .execute(POOL.get().unwrap())
    .await?;

    Ok(())
}

// FETCH ######################################################################

async fn fetch_profile(profile_id: i32) -> Result<Profile> {
    let row = sqlx::query(
        r#"
        select profile_id,
               playlist_id,
               profile_title,
               profile_summary,
               enabled,
               profile_source,
               profile_source_id,
               refresh_interval,
               time_limit,
               track_limit,
               num_sections,
               has_max_sections,
               section_time_limit,
               refreshes_per_hour,
               current_refresh,
               next_refresh_at,
               eligible_for_refresh
        from v_profile
        where profile_id = ?
    "#,
    )
    .bind(profile_id)
    .fetch_one(POOL.get().unwrap())
    .await?;

    let playlist_id = PlexId::try_new(row.try_get::<&str, &str>("playlist_id")?).unwrap();
    let title = Title::try_new(row.try_get::<&str, &str>("profile_title")?).unwrap();
    let profile_source =
        ProfileSource::from_str(row.try_get::<&str, &str>("profile_source")?).unwrap();
    let profile_source_id =
        if let Ok(profile_source_id) = row.try_get::<Option<&str>, &str>("profile_source_id") {
            match profile_source_id {
                Some(id) => Some(ProfileSourceId::try_new(id)?),
                None => None,
            }
        } else {
            None
        };
    let refresh_interval =
        RefreshInterval::try_new(row.try_get::<u32, &str>("refresh_interval")?).unwrap();

    let profile = ProfileBuilder::default()
        .profile_id(row.try_get("profile_id")?)
        .playlist_id(playlist_id)
        .title(title)
        .summary(row.try_get("profile_summary")?)
        .enabled(row.try_get("enabled")?)
        .profile_source(profile_source)
        .profile_source_id(profile_source_id)
        .refresh_interval(refresh_interval)
        .time_limit(row.try_get("time_limit")?)
        .track_limit(row.try_get("track_limit")?)
        // .sections(sections)
        .num_sections(row.try_get("num_sections")?)
        .section_time_limit(row.try_get("section_time_limit")?)
        .refreshes_per_hour(row.try_get("refreshes_per_hour")?)
        .current_refresh(row.try_get("current_refresh")?)
        .next_refresh_at(row.try_get("next_refresh_at")?)
        .eligible_for_refresh(row.try_get("eligible_for_refresh")?)
        .build()
        .unwrap();

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

pub async fn fetch_profiles(enabled: bool) -> Result<Vec<Profile>> {
    let mut sql = r#"
    select profile_id
    from v_profile"#
        .to_string();

    if enabled {
        sql += "\nwhere enabled = 1"
    }
    sql += "\norder by profile_title";

    let ids: Vec<(i32,)> = sqlx::query_as(&sql).fetch_all(POOL.get().unwrap()).await?;

    let mut profiles = vec![];
    for id in ids {
        let profile = fetch_profile(id.0).await?;
        profiles.push(profile)
    }

    Ok(profiles)
}

pub async fn fetch_profile_sections() -> Result<Vec<ProfileSection>> {
    let sections = sqlx::query_as::<_, ProfileSection>("select * from profile_section")
        .fetch_all(POOL.get().unwrap())
        .await?;

    Ok(sections)
}

pub async fn fetch_profile_sections_for_profile(profile_id: i32) -> Result<Vec<ProfileSection>> {
    let sections =
        sqlx::query_as::<_, ProfileSection>("select * from profile_section where profile_id = ?")
            .bind(profile_id)
            .fetch_all(POOL.get().unwrap())
            .await?;

    debug!("{:?}", sections);

    Ok(sections)
}

pub async fn fetch_any_eligible_for_refresh() -> Result<bool> {
    let result: (i32,) = sqlx::query_as(
        r#"
        select count(1) eligible_count
        from v_profile
        where eligible_for_refresh = 1 and enabled = 1;
    "#,
    )
    .fetch_one(POOL.get().unwrap())
    .await?;

    let result = result.0 > 0;

    Ok(result)
}

pub async fn fetch_profiles_to_refresh(force_refresh: bool) -> Result<Vec<Profile>> {
    let mut sql = "select profile_id from v_profile\nwhere".to_string();
    if !force_refresh {
        sql += " eligible_for_refresh = 1 and";
    }
    sql += " enabled = 1";

    let ids: Vec<(i32,)> = sqlx::query_as(&sql).fetch_all(POOL.get().unwrap()).await?;

    let mut profiles = vec![];

    for id in ids {
        let profile = fetch_profile(id.0).await?;
        profiles.push(profile)
    }

    Ok(profiles)
}

pub async fn fetch_profile_by_title(title: &str) -> Result<Option<Profile>> {
    #[allow(dead_code)]
    #[derive(sqlx::FromRow)]
    struct IdTitleResult {
        profile_id: i32,
        profile_title: String,
    }

    let result = sqlx::query_as::<_, IdTitleResult>(
        r#"
        select profile_id, profile_title
        from v_profile
        where profile_title = ?;
    "#,
    )
    .bind(title)
    .fetch_optional(POOL.get().unwrap())
    .await?;

    let profile = if let Some(result) = result {
        let profile = fetch_profile(result.profile_id).await?;
        Some(profile)
    } else {
        None
    };

    Ok(profile)
}

pub async fn fetch_profile_titles() -> Result<Vec<String>> {
    let titles: Vec<(String,)> = sqlx::query_as(
        r#"
        select profile_title from v_profile order by profile_title
    "#,
    )
    .fetch_all(POOL.get().unwrap())
    .await?;

    let titles = titles.into_iter().map(|x| x.0).collect::<Vec<_>>();

    Ok(titles)
}
