use crate::profiles::SectionType;

#[derive(Debug, sqlx::FromRow)]
pub struct DbProfile {
    pub profile_id: i32,
    profile_title: String,
    profile_summary: String,
    enabled: bool,
    profile_source: String,
    profile_source_id: Option<String>,
    refresh_interval: u32,
    time_limit: u32,
    track_limit: u32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct DbProfileSection {
    profile_section_id: i32,
    profile_id: i32,
    section_type: String,
    enabled: bool,
    deduplicate_tracks_by_guid: bool,
    deduplicate_tracks_by_title_and_artist: bool,
    maximum_tracks_by_artist: u32,
    minimum_track_rating: u32,
    randomize_tracks: bool,
    sorting: String,
}