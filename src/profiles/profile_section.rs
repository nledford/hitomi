use std::cmp::Reverse;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use chrono::TimeDelta;
use derive_builder::Builder;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::plex::models::tracks::Track;
use crate::profiles::SectionType;

#[derive(Builder, Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ProfileSection {
    /// Deduplicate tracks by its `guid`, so that the exact same track that appears on
    /// multiple albums (e.g., a studio album and a Greatest Hits album) only appears once in
    /// the resulting playlist.
    deduplicate_tracks_by_guid: bool,
    deduplicate_tracks_by_title_and_artist: bool,
    pub enabled: bool,
    /// Caps the number of tracks by an artist that can appear in a single playlist.
    /// A value of `0` allows for an unlimited number of tracks.
    maximum_tracks_by_artist: u32,
    minimum_track_rating: u32,
    randomize_tracks: bool,
    section_type: SectionType,
    sorting: String,
    #[serde(skip)]
    #[builder(default)]
    tracks: Vec<Track>,
}

impl Display for ProfileSection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut str = format!("  {}", self.section_type);
        str += &format!(
            "\n    Enabled:                                {}",
            self.enabled
        );
        str += &format!(
            "\n    Deduplicate tracks by GUID:             {}",
            self.deduplicate_tracks_by_guid
        );
        str += &format!(
            "\n    Deduplicate tracks by title and artist: {}",
            self.deduplicate_tracks_by_title_and_artist
        );
        str += &format!(
            "\n    Maximum tracks by artist:               {}",
            if self.maximum_tracks_by_artist == 0 {
                "Unlimited".to_string()
            } else {
                format!("{} track(s)", self.maximum_tracks_by_artist)
            }
        );
        str += &format!(
            "\n    Minimum track rating:                   {} stars",
            self.minimum_track_rating
        );
        str += &format!(
            "\n    Sorting:                                {}",
            self.sorting
        );

        writeln!(f, "{str}")
    }
}

impl ProfileSection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_tracks(&self) -> &[Track] {
        &self.tracks
    }

    pub fn set_tracks(&mut self, tracks: Vec<Track>) {
        self.tracks = tracks
    }

    pub fn get_deduplicate_tracks_by_guid(&self) -> bool {
        self.deduplicate_tracks_by_guid
    }

    pub fn get_maximum_tracks_by_artist(&self) -> u32 {
        self.maximum_tracks_by_artist
    }

    pub fn get_minimum_track_rating(&self) -> u32 {
        if self.minimum_track_rating <= 1 {
            return 0;
        }
        (self.minimum_track_rating - 1) * 2
    }

    pub fn get_sorting(&self) -> Vec<&str> {
        self.sorting.split(',').collect::<_>()
    }

    pub fn is_unplayed(&self) -> bool {
        self.section_type == SectionType::Unplayed
    }

    pub fn is_least_played(&self) -> bool {
        self.section_type == SectionType::LeastPlayed
    }

    pub fn is_oldest(&self) -> bool {
        self.section_type == SectionType::Oldest
    }

    pub fn num_tracks(&self) -> usize {
        self.tracks.len()
    }

    pub fn run_manual_filters(&mut self, time_limit: f64, list_to_dedup: Option<&mut Vec<Track>>) {
        self.deduplicate_by_track_guid();
        self.run_deduplicate_by_title_and_artist();
        self.limit_tracks_by_artist();
        self.sort_tracks();
        self.reduce_to_time_limit(time_limit);

        if let Some(lst) = list_to_dedup {
            self.dedup_tracks_by_list(lst)
        }

        if self.randomize_tracks {
            self.tracks.shuffle(&mut rand::thread_rng())
        }
    }

    fn deduplicate_by_track_guid(&mut self) {
        if self.deduplicate_tracks_by_guid {
            self.tracks
                .dedup_by_key(|track| track.get_guid().to_owned());
        }
    }

    fn run_deduplicate_by_title_and_artist(&mut self) {
        if self.deduplicate_tracks_by_title_and_artist {
            self.tracks
                .sort_by_key(|track| (track.title().to_owned(), track.artist().to_owned()));
            self.tracks
                .dedup_by_key(|track| (track.title().to_owned(), track.artist().to_owned()));
        }
    }

    fn limit_tracks_by_artist(&mut self) {
        if self.maximum_tracks_by_artist == 0 {
            return;
        }

        if self.is_unplayed() || self.is_least_played() {
            self.tracks
                .sort_by_key(|track| (track.plays(), track.last_played()))
        } else {
            self.tracks
                .sort_by_key(|track| (track.last_played(), track.plays()))
        }

        let mut artist_occurrences: HashMap<String, u32> = HashMap::new();
        self.tracks.retain(|track| {
            let artist_guid = track.get_artist_guid().to_owned();
            let occurrences = artist_occurrences.entry(artist_guid).or_insert(0);
            *occurrences += 1;

            *occurrences <= self.maximum_tracks_by_artist
        })
    }

    fn sort_tracks(&mut self) {
        if self.is_unplayed() {
            self.tracks
                .sort_by_key(|t| (Reverse(t.rating()), t.plays(), t.last_played()))
        }
        if self.is_least_played() {
            self.tracks.sort_by_key(|t| (t.plays(), t.last_played()))
        }
        if self.is_oldest() {
            self.tracks.sort_by_key(|t| (t.last_played(), t.plays()))
        }
    }

    fn dedup_tracks_by_list(&mut self, comp: &[Track]) {
        self.tracks.retain(|t| !comp.contains(t))
    }

    pub fn reduce_to_time_limit(&mut self, time_limit: f64) {
        let limit = TimeDelta::seconds((time_limit * 60_f64 * 60_f64) as i64);

        let total_duration: i64 = self.get_tracks().iter().map(|track| track.duration()).sum();
        let total_duration = TimeDelta::milliseconds(total_duration);

        if total_duration <= limit {
            return;
        }

        let mut accum_total = TimeDelta::seconds(0);
        let index = self
            .get_tracks()
            .iter()
            .position(|track| {
                accum_total += TimeDelta::milliseconds(track.duration());
                accum_total > limit
            })
            .unwrap_or(0);

        self.set_tracks(self.tracks[..=index].to_vec())
    }
}

// TESTS ######################################################################
