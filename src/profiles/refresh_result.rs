use std::fmt::{Display, Formatter};
use std::time;
use std::time::Duration;

use crate::plex::models::tracks::Track;
use crate::profiles::ProfileAction;

pub struct RefreshResult {
    profile_title: String,
    tracks: Vec<Track>,
    action: ProfileAction,
}

impl RefreshResult {
    pub fn new(profile_title: &str, tracks: &[Track], action: ProfileAction) -> RefreshResult {
        Self {
            profile_title: profile_title.to_string(),
            tracks: tracks.to_vec(),
            action,
        }
    }

    pub fn get_title(&self) -> String {
        self.profile_title.clone()
    }

    fn get_size(&self) -> usize {
        self.tracks.len()
    }

    /// In Milliseconds
    fn get_total_duration(&self) -> i64 {
        self.tracks.iter().map(|t| t.get_track_duration()).sum()
    }

    fn get_duration(&self) -> Duration {
        Duration::from_millis(self.get_total_duration() as u64)
    }

    fn get_avg_track_duration(&self) -> String {
        let avg_track_duration =
            (self.get_total_duration() as f64 / self.get_size() as f64).floor() as u64;
        let avg_track_duration = time::Duration::from_millis(avg_track_duration);
        humantime::format_duration(avg_track_duration).to_string()
    }

    fn get_duration_str(&self) -> String {
        humantime::format_duration(self.get_duration()).to_string()
    }

    fn get_action(&self) -> &str {
        if self.action == ProfileAction::Create {
            "created"
        } else {
            "updated"
        }
    }
}

impl Display for RefreshResult {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let mut str = String::new();
        str += &format!(
            "Successfully {} `{}` playlist!",
            self.get_action(),
            self.profile_title
        );
        str += &format!("\n  Final size:                {} tracks", self.get_size());
        str += &format!("\n  Final total duration:      {}", self.get_duration_str());
        str += &format!(
            "\n  Average track duration:    {}",
            self.get_avg_track_duration()
        );

        write!(f, "{str}")
    }
}
