use crate::plex::models::tracks::Track;
use crate::profiles;
use crate::profiles::profile_section::ProfileSection;
use anyhow::Result;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct SectionFetchResult {
    unplayed: Vec<Track>,
    least_played: Vec<Track>,
    oldest: Vec<Track>,
}

#[derive(Builder, Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Sections {
    unplayed_section: Option<ProfileSection>,
    least_played_section: Option<ProfileSection>,
    oldest_section: Option<ProfileSection>,
}

impl Default for Sections {
    fn default() -> Self {
        Self {
            unplayed_section: Some(ProfileSection::default()),
            least_played_section: Some(ProfileSection::default()),
            oldest_section: Some(ProfileSection::default()),
        }
    }
}

impl Sections {
    pub fn has_unplayed_section(&self) -> bool {
        if let Some(section) = &self.unplayed_section {
            section.is_enabled()
        } else {
            false
        }
    }

    pub fn has_least_played_section(&self) -> bool {
        if let Some(section) = &self.least_played_section {
            section.is_enabled()
        } else {
            false
        }
    }

    pub fn has_oldest_section(&self) -> bool {
        if let Some(section) = &self.oldest_section {
            section.is_enabled()
        } else {
            false
        }
    }

    pub fn set_unplayed_section(&mut self, section: Option<ProfileSection>) {
        self.unplayed_section = section
    }

    pub fn set_least_played_section(&mut self, section: Option<ProfileSection>) {
        self.least_played_section = section
    }

    pub fn set_oldest_section(&mut self, section: Option<ProfileSection>) {
        self.oldest_section = section
    }

    fn set_unplayed_tracks(&mut self, tracks: Vec<Track>, time_limit: f64) {
        if let Some(section) = &mut self.unplayed_section {
            section.set_tracks(tracks);
            section.run_manual_filters(time_limit, None);
        }
    }

    fn set_least_played_tracks(&mut self, tracks: Vec<Track>, time_limit: f64) {
        if let Some(section) = &mut self.least_played_section {
            section.set_tracks(tracks);
            section.run_manual_filters(time_limit, None)
        }
    }

    fn set_oldest_tracks(&mut self, tracks: Vec<Track>, time_limit: f64) {
        if let Some(section) = &mut self.oldest_section {
            section.set_tracks(tracks);
            section.run_manual_filters(time_limit, None);
        }
    }

    pub fn num_enabled(&self) -> i32 {
        [
            self.has_unplayed_section(),
            self.has_least_played_section(),
            self.has_oldest_section(),
        ]
        .into_iter()
        .filter(|x| *x)
        .count() as i32
    }

    pub async fn fetch_tracks(
        &self,
        profile_title: &str,
        limit: Option<i32>,
    ) -> Result<SectionFetchResult> {
        let unplayed =
            profiles::fetch_section_tracks(self.get_unplayed_section(), profile_title, limit)
                .await?;
        let least_played =
            profiles::fetch_section_tracks(self.get_least_played_section(), profile_title, limit)
                .await?;
        let oldest =
            profiles::fetch_section_tracks(self.get_oldest_section(), profile_title, limit).await?;

        Ok(SectionFetchResult {
            unplayed,
            least_played,
            oldest,
        })
    }

    pub async fn set_tracks(&mut self, tracks: SectionFetchResult, time_limit: f64) {
        self.set_unplayed_tracks(tracks.unplayed, time_limit);
        self.set_least_played_tracks(tracks.least_played, time_limit);
        self.set_oldest_tracks(tracks.oldest, time_limit);
    }

    pub fn get_unplayed_section(&self) -> Option<&ProfileSection> {
        self.unplayed_section.as_ref()
    }

    pub fn get_unplayed_tracks(&self) -> Option<&[Track]> {
        if let Some(section) = self.get_unplayed_section() {
            Some(section.get_tracks())
        } else {
            None
        }
    }

    pub fn num_unplayed_tracks(&self) -> usize {
        if let Some(tracks) = self.get_unplayed_tracks() {
            tracks.len()
        } else {
            0
        }
    }

    pub fn get_least_played_section(&self) -> Option<&ProfileSection> {
        self.least_played_section.as_ref()
    }

    pub fn get_least_played_tracks(&self) -> Option<&[Track]> {
        if let Some(section) = self.get_least_played_section() {
            Some(section.get_tracks())
        } else {
            None
        }
    }

    pub fn num_least_played_tracks(&self) -> usize {
        if let Some(tracks) = self.get_least_played_tracks() {
            tracks.len()
        } else {
            0
        }
    }

    pub fn get_oldest_section(&self) -> Option<&ProfileSection> {
        self.oldest_section.as_ref()
    }

    pub fn get_oldest_tracks(&self) -> Option<&[Track]> {
        if let Some(section) = self.get_oldest_section() {
            Some(section.get_tracks())
        } else {
            None
        }
    }

    pub fn num_oldest_tracks(&self) -> usize {
        if let Some(tracks) = self.get_oldest_tracks() {
            tracks.len()
        } else {
            0
        }
    }

    pub fn global_track_total(&self) -> usize {
        self.num_unplayed_tracks() + self.num_least_played_tracks() + self.num_oldest_tracks()
    }
}
