use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};

use anyhow::Result;
use chrono::{Local, NaiveDateTime};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::db;
use crate::plex::types::PlexId;
use crate::profiles::profile_section::ProfileSection;
use crate::profiles::types::{ProfileSourceId, RefreshInterval};
use crate::profiles::ProfileSource;
use crate::types::Title;

// PROFILE ####################################################################

#[derive(Builder, Clone, Debug, Default, Deserialize, Serialize, PartialEq, sqlx::FromRow)]
#[builder(default)]
pub struct Profile {
    /// The primary key in the database
    profile_id: i32,
    /// The plex ID for the playlist
    playlist_id: PlexId,
    /// The name of the profile and the resulting playlist
    title: Title,
    /// The summary for the profile and the resulting playlist
    summary: String,
    /// Indicates whether to use the profile. If false, the application will skip this profile when
    /// refreshing playlists
    #[builder(default = "true")]
    enabled: bool,
    /// The location from which the profile fetches tracks
    profile_source: ProfileSource,
    profile_source_id: Option<ProfileSourceId>,
    /// How often in minutes the profile should refresh in an hour
    refresh_interval: RefreshInterval,
    /// The time limit in hours of the playlist.
    time_limit: u32,
    /// The track limit of the playlist
    track_limit: u32,
    #[builder(default)]
    num_sections: u32,
    #[builder(default)]
    section_time_limit: f64,
    #[builder(default)]
    refreshes_per_hour: u32,
    #[builder(default)]
    current_refresh: NaiveDateTime,
    #[builder(default)]
    next_refresh_at: NaiveDateTime,
    #[builder(default)]
    eligible_for_refresh: bool,
}

impl Profile {
    pub fn get_profile_id(&self) -> i32 {
        self.profile_id
    }

    pub fn get_playlist_id(&self) -> &PlexId {
        &self.playlist_id
    }

    pub fn set_playlist_id(&mut self, playlist_id: PlexId) {
        self.playlist_id = playlist_id
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn get_summary(&self) -> &str {
        &self.summary
    }

    pub fn get_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_profile_source(&self) -> &ProfileSource {
        &self.profile_source
    }

    pub fn get_profile_source_id(&self) -> Option<&ProfileSourceId> {
        self.profile_source_id.as_ref()
    }

    pub fn get_profile_source_id_str(&self) -> Option<&str> {
        if let Some(id) = &self.profile_source_id {
            Some(id.as_ref())
        } else {
            None
        }
    }

    pub async fn fetch_sections(&self) -> Result<Vec<ProfileSection>> {
        let sections = db::profiles::fetch_profile_sections_for_profile(self.profile_id).await?;
        Ok(sections)
    }

    pub fn get_refresh_interval(&self) -> &u32 {
        self.refresh_interval.as_ref()
    }

    pub fn get_time_limit(&self) -> u32 {
        self.time_limit
    }

    pub fn get_track_limit(&self) -> u32 {
        self.track_limit
    }

    pub fn get_section_time_limit(&self) -> f64 {
        self.section_time_limit
    }

    pub fn get_refreshes_per_hour(&self) -> u32 {
        self.refreshes_per_hour
    }

    pub fn check_for_refresh(&self, force_refresh: bool) -> bool {
        if force_refresh {
            return true;
        }
        self.eligible_for_refresh
    }

    pub fn get_next_refresh_hour_minute(&self) -> String {
        self.next_refresh_at.format("%R").to_string()
    }

    pub fn get_next_refresh_str(&self) -> String {
        format!(
            "LAST UPDATE: {}\nNEXT UPDATE: {}",
            Local::now().format("%F %T"),
            self.next_refresh_at.format("%R")
        )
    }

    pub fn get_profile_source_and_id(&self) -> (&ProfileSource, Option<&ProfileSourceId>) {
        (self.get_profile_source(), self.get_profile_source_id())
    }

    fn refresh_interval_str(&self) -> String {
        format!(
            "Every {} minutes ({} refreshes per hour)",
            self.refresh_interval, self.refreshes_per_hour
        )
    }

    fn time_limit_str(&self) -> String {
        if self.time_limit == 0 {
            "No Limit".to_string()
        } else {
            format!("{} hours", self.time_limit)
        }
    }

    fn get_track_limit_str(&self) -> String {
        if self.track_limit == 0 {
            "No Limit".to_string()
        } else {
            format!("{} tracks", self.track_limit)
        }
    }
}

impl Display for Profile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut str = format!("\n{}", self.title);
        str += &format!("\n{}", self.summary);
        str += &format!("\nEnabled:          {}", self.enabled);
        str += &format!("\nSource:           {}", self.profile_source);
        str += &format!("\nRefresh Interval: {}", self.refresh_interval_str());
        str += &format!("\nTime Limit:       {}", self.time_limit_str());
        str += &format!("\nTrack Limit:      {}", self.get_track_limit_str());

        // TODO fix sections info
        str += "\n\nSections:";
        // if self.has_unplayed_tracks() {
        //     str += &format!("\n{}", self.sections.iter().find())
        // }
        //
        // if self.has_least_played_tracks() {
        //     str += &format!("\n{}", self.sections.get_least_played_section().unwrap())
        // }
        //
        // if self.has_oldest_tracks() {
        //     str += &format!("\n{}", self.sections.get_oldest_section().unwrap())
        // }

        write!(f, "{str}")
    }
}
