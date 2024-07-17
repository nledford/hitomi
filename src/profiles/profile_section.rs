use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Display, Formatter};

use chrono::TimeDelta;
use derive_builder::Builder;
use itertools::Itertools;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use simplelog::info;

use crate::plex::models::tracks::Track;
use crate::profiles::SectionType;

#[allow(dead_code)]
#[derive(Builder, Clone, Debug, Default, Deserialize, PartialEq, Serialize, sqlx::FromRow)]
pub struct ProfileSection {
    /// The primary key in the database
    #[builder(setter(skip))]
    profile_section_id: i32,
    /// The foreign key linking to the profile in the database
    #[builder(setter(skip))]
    profile_id: i32,
    /// Deduplicate tracks by its `guid`, so that the exact same track that appears on
    /// multiple albums (e.g., a studio album and a Greatest Hits album) only appears once in
    /// the resulting playlist.
    deduplicate_tracks_by_guid: bool,
    deduplicate_tracks_by_title_and_artist: bool,
    enabled: bool,
    /// Caps the number of tracks by an artist that can appear in a single playlist.
    /// A value of `0` allows for an unlimited number of tracks.
    maximum_tracks_by_artist: u32,
    minimum_track_rating: u32,
    randomize_tracks: bool,
    section_type: SectionType,
    sorting: String,
}

impl ProfileSection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_profile_section_id(&self) -> i32 {
        self.profile_section_id
    }

    pub fn get_profile_id(&self) -> i32 {
        self.profile_id
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_section_type(&self) -> SectionType {
        self.section_type
    }

    pub fn is_section_type(&self, section_type: SectionType) -> bool {
        self.get_section_type() == section_type
    }

    pub fn is_unplayed_section(&self) -> bool {
        self.is_section_type(SectionType::Unplayed)
    }

    pub fn is_least_played_section(&self) -> bool {
        self.is_section_type(SectionType::LeastPlayed)
    }

    pub fn is_oldest_section(&self) -> bool {
        self.is_section_type(SectionType::Oldest)
    }

    pub fn get_minimum_track_rating(&self) -> u32 {
        if self.minimum_track_rating <= 1 {
            return 0;
        }
        self.minimum_track_rating
    }

    pub fn get_minimum_track_rating_adjusted(&self) -> u32 {
        if self.get_minimum_track_rating() <= 1 {
            return 0;
        }
        (self.get_minimum_track_rating() - 1) * 2
    }

    pub fn get_sorting_vec(&self) -> Vec<&str> {
        self.sorting.split(',').collect::<_>()
    }

    pub fn get_sorting(&self) -> &str {
        &self.sorting
    }

    pub fn get_deduplicate_tracks_by_guid(&self) -> bool {
        self.deduplicate_tracks_by_guid
    }

    pub fn get_deduplicate_tracks_by_title_and_artist(&self) -> bool {
        self.deduplicate_tracks_by_title_and_artist
    }

    pub fn get_maximum_tracks_by_artist(&self) -> u32 {
        self.maximum_tracks_by_artist
    }

    pub fn get_randomize_tracks(&self) -> bool {
        self.randomize_tracks
    }

    pub fn run_manual_filters(
        &self,
        tracks: &mut Vec<Track>,
        section_type: SectionType,
        time_limit: f64,
    ) {
        info!("Running manual section filters...");

        self.deduplicate_by_track_guid(tracks);
        self.run_deduplicate_by_title_and_artist(tracks);
        self.limit_tracks_by_artist(tracks);
        self.sort_tracks(tracks);
        self.reduce_to_time_limit(tracks, time_limit);
        self.track_randomizer(tracks, section_type);
    }

    fn track_randomizer(&self, tracks: &mut Vec<Track>, section_type: SectionType) {
        if !self.randomize_tracks {
            return;
        }

        if section_type != SectionType::Oldest {
            *tracks = tracks
                .iter()
                .sorted_by_key(|track| (track.plays(), track.last_played()))
                .fold(
                    BTreeMap::new(),
                    |mut acc: BTreeMap<i32, Vec<Track>>, track| {
                        let plays = acc.entry(track.plays()).or_default();
                        plays.push(track.clone());
                        acc
                    },
                )
                .iter_mut()
                .fold(Vec::new(), |mut acc, (_, group)| {
                    group.shuffle(&mut rand::thread_rng());
                    acc.append(group);
                    acc
                })
        } else {
            tracks.shuffle(&mut rand::thread_rng())
        }
    }

    fn deduplicate_by_track_guid(&self, tracks: &mut Vec<Track>) {
        if self.deduplicate_tracks_by_guid {
            tracks.dedup_by_key(|track| track.get_guid().to_owned());
        }
    }

    fn run_deduplicate_by_title_and_artist(&self, tracks: &mut Vec<Track>) {
        if self.deduplicate_tracks_by_title_and_artist {
            tracks.sort_by_key(|track| (track.title().to_owned(), track.artist().to_owned()));
            tracks.dedup_by_key(|track| (track.title().to_owned(), track.artist().to_owned()));
        }
    }

    fn limit_tracks_by_artist(&self, tracks: &mut Vec<Track>) {
        if self.maximum_tracks_by_artist == 0 {
            return;
        }

        if self.is_unplayed_section() || self.is_least_played_section() {
            tracks.sort_by_key(|track| (track.plays(), track.last_played()))
        } else {
            tracks.sort_by_key(|track| (track.last_played(), track.plays()))
        }

        let mut artist_occurrences: HashMap<String, u32> = HashMap::new();
        tracks.retain(|track| {
            let artist_guid = track.get_artist_guid().to_owned();
            let occurrences = artist_occurrences.entry(artist_guid).or_insert(0);
            *occurrences += 1;

            *occurrences <= self.maximum_tracks_by_artist
        })
    }

    fn sort_tracks(&self, tracks: &mut [Track]) {
        if self.is_unplayed_section() {
            tracks.sort_by_key(|t| (Reverse(t.rating()), t.plays(), t.last_played()))
        }
        if self.is_least_played_section() {
            tracks.sort_by_key(|t| (t.plays(), t.last_played()))
        }
        if self.is_oldest_section() {
            tracks.sort_by_key(|t| (t.last_played(), t.plays()))
        }
    }

    pub fn reduce_to_time_limit(&self, tracks: &mut Vec<Track>, time_limit: f64) {
        let limit = TimeDelta::seconds((time_limit * 60_f64 * 60_f64) as i64);

        let total_duration: i64 = tracks.iter().map(|track| track.duration()).sum();
        let total_duration = TimeDelta::milliseconds(total_duration);

        if total_duration <= limit {
            return;
        }

        let mut accum_total = TimeDelta::seconds(0);
        let index = tracks
            .iter()
            .position(|track| {
                accum_total += TimeDelta::milliseconds(track.duration());
                accum_total > limit
            })
            .unwrap_or(0);

        *tracks = tracks[..=index].to_vec();
    }
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
