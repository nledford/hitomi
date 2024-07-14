use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::path::PathBuf;

use crate::plex::types::PlexId;
use crate::profiles::profile_section::ProfileSection;
use crate::profiles::types::{ProfileSourceId, RefreshInterval};
use crate::profiles::ProfileSource;
use crate::types::Title;
use crate::utils;
use chrono::{DateTime, Local, TimeDelta, Timelike};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

// PROFILE ####################################################################

#[derive(Builder, Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[builder(default)]
pub struct Profile {
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
    /// Profile sections
    sections: Vec<ProfileSection>,
}

impl Profile {
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

    fn file_name(&self) -> String {
        format!("{}.json", self.title)
    }

    pub fn get_sections(&self) -> &[ProfileSection] {
        &self.sections
    }

    pub async fn get_profile_path(&self, profiles_directory: &str) -> PathBuf {
        PathBuf::new()
            .join(profiles_directory)
            .join(self.file_name())
    }

    pub fn check_for_refresh(&self, force_refresh: bool) -> bool {
        if force_refresh {
            return true;
        }

        let current_minute = Local::now().minute();
        let matches_top_of_the_hour = current_minute == 0;
        let matches_refresh_minute = current_minute == self.get_current_refresh_time().minute();

        matches_top_of_the_hour || matches_refresh_minute
    }

    fn get_current_refresh_time(&self) -> DateTime<Local> {
        let current_minute = utils::build_refresh_minutes(&self.refresh_interval)
            .into_iter()
            .find(|x| *x >= Local::now().minute())
            .unwrap_or(0);

        Local::now()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .add(TimeDelta::minutes(current_minute as i64))
    }

    pub fn get_next_refresh_time(&self) -> DateTime<Local> {
        let next_minute = utils::build_refresh_minutes(&self.refresh_interval)
            .into_iter()
            .find(|x| *x > Local::now().minute())
            .unwrap_or(0);

        Local::now()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .add(TimeDelta::minutes(next_minute as i64))
    }

    pub fn get_next_refresh_hour_minute(&self) -> String {
        self.get_next_refresh_time().format("%R").to_string()
    }

    pub fn get_next_refresh_str(&self) -> String {
        let next_refresh_time = self.get_next_refresh_time();
        format!(
            "LAST UPDATE: {}\nNEXT UPDATE: {}",
            Local::now().format("%F %T"),
            next_refresh_time.format("%R")
        )
    }

    pub fn get_profile_source_and_id(&self) -> (&ProfileSource, Option<&ProfileSourceId>) {
        (self.get_profile_source(), self.get_profile_source_id())
    }

    fn refresh_interval_str(&self) -> String {
        format!(
            "Every {} minutes ({} refreshes per hour)",
            self.refresh_interval,
            self.refreshes_per_hour()
        )
    }

    fn refreshes_per_hour(&self) -> i32 {
        60 / *self.refresh_interval.as_ref() as i32
    }

    fn time_limit_str(&self) -> String {
        if self.time_limit == 0 {
            "No Limit".to_string()
        } else {
            format!("{} hours", self.time_limit)
        }
    }

    pub fn get_section_time_limit(&self) -> f64 {
        self.time_limit as f64
            / self
            .sections
            .iter()
            .filter(|section| section.is_enabled())
            .count() as f64
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
