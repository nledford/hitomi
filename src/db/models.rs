use crate::profiles::SectionType;

#[derive(sqlx::FromRow)]
pub struct DbProfile {
    pub profile_id: i32,
    pub playlist_id: String,
    pub profile_title: String,
    pub profile_summary: String,
    pub enabled: bool,
    pub profile_source: String,
    pub profile_source_id: Option<String>,
    pub refresh_interval: u32,
    pub time_limit: u32,
    pub track_limit: u32,
}

#[derive(sqlx::FromRow)]
pub struct DbProfileSection {
    profile_section_id: i32,
    profile_id: i32,
    pub section_type: SectionType,
    pub enabled: bool,
    pub deduplicate_tracks_by_guid: bool,
    pub deduplicate_tracks_by_title_and_artist: bool,
    pub maximum_tracks_by_artist: u32,
    pub minimum_track_rating: u32,
    pub randomize_tracks: bool,
    pub sorting: String,
}

