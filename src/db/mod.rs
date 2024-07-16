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

/*fn db_profile_to_profile(db_profile: DbProfile, db_sections: Vec<DbProfileSection>) -> Profile {
    let sections = db_sections
        .into_iter()
        .map(|db_section| {
            ProfileSectionBuilder::default()
                .section_type(db_section.section_type)
                .enabled(db_section.enabled)
                .deduplicate_tracks_by_title_and_artist(
                    db_section.deduplicate_tracks_by_title_and_artist,
                )
                .deduplicate_tracks_by_guid(db_section.deduplicate_tracks_by_guid)
                .maximum_tracks_by_artist(db_section.maximum_tracks_by_artist)
                .minimum_track_rating(db_section.minimum_track_rating)
                .randomize_tracks(db_section.randomize_tracks)
                .sorting(db_section.sorting)
                .build()
                .unwrap()
        })
        .collect::<Vec<_>>();

    let profile_source_id = if let Some(id) = db_profile.profile_source_id {
        if let Ok(profile_source_id) = ProfileSourceId::try_new(id) {
            Some(profile_source_id)
        } else {
            None
        }
    } else {
        None
    };

    let profile = ProfileBuilder::default()
        .playlist_id(PlexId::try_new(db_profile.playlist_id).unwrap())
        .title(Title::try_new(db_profile.profile_title).unwrap())
        .summary(db_profile.profile_summary)
        .enabled(db_profile.enabled)
        .profile_source(ProfileSource::from_str(&db_profile.profile_source).unwrap())
        .profile_source_id(profile_source_id)
        .refresh_interval(RefreshInterval::try_new(db_profile.refresh_interval).unwrap())
        .time_limit(db_profile.time_limit)
        .track_limit(db_profile.track_limit)
        .sections(sections)
        .build()
        .unwrap();

    profile
}*/
