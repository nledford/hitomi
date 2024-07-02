use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::plex::models::tracks::Track;
use crate::profiles;
use crate::profiles::profile::Profile;
use crate::profiles::profile_section::ProfileSection;
use crate::state::AppState;

#[derive(Builder, Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Sections {
    unplayed_tracks: ProfileSection,
    least_played_tracks: ProfileSection,
    oldest_tracks: ProfileSection,
}

impl Sections {
    pub fn has_unplayed_tracks(&self) -> bool {
        self.unplayed_tracks.enabled
    }

    pub fn has_least_played_tracks(&self) -> bool {
        self.least_played_tracks.enabled
    }

    pub fn has_oldest_tracks(&self) -> bool {
        self.oldest_tracks.enabled
    }

    pub fn set_unplayed_tracks(&mut self, section: ProfileSection) {
        self.unplayed_tracks = section
    }

    pub fn set_least_played_tracks(&mut self, section: ProfileSection) {
        self.least_played_tracks = section
    }

    pub fn set_oldest_tracks(&mut self, section: ProfileSection) {
        self.oldest_tracks = section
    }

    pub fn num_enabled(&self) -> i32 {
        [
            self.unplayed_tracks.enabled,
            self.least_played_tracks.enabled,
            self.oldest_tracks.enabled,
        ]
        .into_iter()
        .filter(|x| *x)
        .count() as i32
    }

    pub async fn fetch_tracks(
        &mut self,
        profile: &Profile,
        app_state: &AppState,
        limit: Option<i32>,
    ) -> anyhow::Result<()> {
        profiles::fetch_section_tracks(&mut self.unplayed_tracks, profile, app_state, limit)
            .await?;
        profiles::fetch_section_tracks(&mut self.least_played_tracks, profile, app_state, limit)
            .await?;
        profiles::fetch_section_tracks(&mut self.oldest_tracks, profile, app_state, limit).await?;

        Ok(())
    }

    pub fn get_unplayed_section(&self) -> &ProfileSection {
        &self.unplayed_tracks
    }

    pub fn get_unplayed_tracks(&self) -> &[Track] {
        self.unplayed_tracks.get_tracks()
    }

    fn num_unplayed_tracks(&self) -> usize {
        self.unplayed_tracks.num_tracks()
    }

    pub fn get_least_played_section(&self) -> &ProfileSection {
        &self.least_played_tracks
    }

    pub fn get_least_played_tracks(&self) -> &[Track] {
        self.least_played_tracks.get_tracks()
    }

    fn num_least_played_tracks(&self) -> usize {
        self.least_played_tracks.num_tracks()
    }

    pub fn get_oldest_section(&self) -> &ProfileSection {
        &self.oldest_tracks
    }

    pub fn get_oldest_tracks(&self) -> &[Track] {
        self.oldest_tracks.get_tracks()
    }

    fn num_oldest_tracks(&self) -> usize {
        self.oldest_tracks.num_tracks()
    }

    pub fn global_track_total(&self) -> usize {
        self.num_unplayed_tracks() + self.num_least_played_tracks() + self.num_oldest_tracks()
    }
}
