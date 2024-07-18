use std::cmp::Reverse;
use std::collections::BTreeMap;

use chrono::TimeDelta;
use derive_builder::Builder;
use rand::seq::SliceRandom;
use simplelog::info;

use crate::plex::models::tracks::Track;
use crate::profiles::profile_section::ProfileSection;
use crate::profiles::SectionType;

#[derive(Builder, Debug, Default)]
pub struct SectionTracksMerger {
    #[builder(default)]
    unplayed: Vec<Track>,
    #[builder(default)]
    least_played: Vec<Track>,
    #[builder(default)]
    oldest: Vec<Track>,
    #[builder(default)]
    merged: Vec<Track>,
}

impl SectionTracksMerger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_unplayed_tracks(&mut self, tracks: Vec<Track>) {
        self.unplayed = tracks
    }

    pub fn set_least_played_tracks(&mut self, tracks: Vec<Track>) {
        self.least_played = tracks
    }

    pub fn set_oldest_tracks(&mut self, tracks: Vec<Track>) {
        self.oldest = tracks
    }

    /// Runs manual filters for the profile sections
    ///
    /// Manual filters are those that are unique to this application and not included with plex
    pub fn run_manual_filters(&mut self, profile_sections: &[ProfileSection], time_limit: f64) {
        info!("Running manual section filters...");
        self.deduplicate_lists();

        for section in profile_sections {
            let tracks = match section.get_section_type() {
                SectionType::Unplayed => &mut self.unplayed,
                SectionType::LeastPlayed => &mut self.least_played,
                SectionType::Oldest => &mut self.oldest,
            };

            if section.get_deduplicate_tracks_by_guid() {
                deduplicate_by_track_guid(tracks);
            }

            if section.get_deduplicate_tracks_by_title_and_artist() {
                deduplicate_by_title_and_artist(tracks);
            }

            trim_tracks_by_artist(
                tracks,
                section.get_maximum_tracks_by_artist(),
                section.get_section_type(),
            );

            sort_tracks(tracks, section.get_section_type());
            reduce_to_time_limit(tracks, time_limit);
            if section.get_randomize_tracks() {
                randomizer(tracks, section.get_section_type())
            }
        }
    }

    /// Deduplicates the least played and oldest tracks
    ///
    /// Least played is deduplicated first, and oldest is deduplicated second
    fn deduplicate_lists(&mut self) {
        deduplicate_tracks_by_lists(&mut self.least_played, vec![&self.oldest]);
        deduplicate_tracks_by_lists(&mut self.oldest, vec![&self.least_played]);
    }

    /// Returns a slice of the merged tracks
    pub fn get_combined_tracks(&self) -> &[Track] {
        &self.merged
    }

    /// Returns `false` if no sections are valid
    fn none_are_valid(&self) -> bool {
        self.get_num_valid() == 0
    }

    /// Returns the number of valid sections (those that are not empty)
    fn get_num_valid(&self) -> usize {
        [
            !self.unplayed.is_empty(),
            !self.least_played.is_empty(),
            !self.oldest.is_empty(),
        ]
            .iter()
            .filter(|x| **x)
            .count()
    }

    /// Calculates the largest section from all sections included in the merger
    ///
    /// # Example
    ///
    /// - Unplayed Tracks:      100 Tracks
    /// - Least Played Tracks:  105 Tracks
    /// - Oldest Track:         103 Tracks
    ///
    /// The largest section is Least Played Tracks
    fn get_largest_section_length(&self) -> usize {
        *[
            self.unplayed.len(),
            self.least_played.len(),
            self.oldest.len(),
        ]
            .iter()
            .max()
            .unwrap_or(&0_usize)
    }

    /// Returns a [`Vec`] of track IDs
    pub fn get_track_ids(&self) -> Vec<String> {
        if self.merged.is_empty() {
            vec![]
        } else {
            self.merged
                .iter()
                .map(|track| track.id().to_string())
                .collect::<Vec<_>>()
        }
    }

    /// Displays the first 25 tracks in the merged playlist in the console
    pub fn print_preview(&self) {
        if self.merged.is_empty() {
            return;
        }

        let preview = self.merged.iter().take(25).collect::<Vec<_>>();

        for (i, track) in preview.iter().enumerate() {
            println!("{:2} {}", i + 1, track)
        }
    }

    /// Merges tracks from each playlist section into a single playlist
    ///
    /// The following pattern is followed:
    ///  - Unplayed
    ///  - Least Played
    ///  - Oldest
    ///
    /// If a track cannot be found in a given section, that section is skipped.
    pub fn merge(&mut self) {
        if self.none_are_valid() {
            return;
        }
        info!(
            "Merging {} section{}...",
            self.get_num_valid(),
            if self.get_num_valid() == 1 { "" } else { "s" }
        );

        self.merged = Vec::new();
        for i in 0..self.get_largest_section_length() {
            if let Some(track) = self.unplayed.get(i) {
                self.merged.push(track.clone())
            }

            if let Some(track) = self.least_played.get(i) {
                self.merged.push(track.clone())
            }

            if let Some(track) = self.oldest.get(i) {
                self.merged.push(track.clone())
            }
        }
    }
}

/// Remove duplicate tracks by the title and artist of a track
///
/// e,g, If the track "The Beatles - Get Back" appears multiple times in a playlist, any duplicates will be removed.
fn deduplicate_by_title_and_artist(tracks: &mut Vec<Track>) {
    tracks.sort_by_key(|track| (track.title().to_owned(), track.artist().to_owned()));
    tracks.dedup_by_key(|track| (track.title().to_owned(), track.artist().to_owned()));
}

/// Remove duplicate tracks based on the Plex `GUID`
fn deduplicate_by_track_guid(tracks: &mut Vec<Track>) {
    tracks.dedup_by_key(|track| track.get_guid().to_owned());
}

/// Deduplicates one list based on values in other lists
fn deduplicate_tracks_by_lists(tracks: &mut Vec<Track>, lists: Vec<&[Track]>) {
    for list in lists {
        tracks.retain(|t| !list.contains(t))
    }
}

/// Trims tracks by artist limit (in other words, the maximum number of tracks that can be included in the list by a single artist)
///
/// Returns early if the limit is zero
fn trim_tracks_by_artist(
    tracks: &mut Vec<Track>,
    maximum_tracks_by_artist: u32,
    section_type: SectionType,
) {
    if maximum_tracks_by_artist == 0 {
        return;
    }

    match section_type {
        SectionType::Oldest => tracks.sort_by_key(|track| (track.last_played(), track.plays())),
        _ => tracks.sort_by_key(|track| (track.plays(), track.last_played())),
    }

    let mut artist_occurrences: BTreeMap<String, u32> = BTreeMap::new();
    tracks.retain(|track| {
        let artist_guid = track.get_artist_guid().to_owned();
        let occurrences = artist_occurrences.entry(artist_guid).or_default();
        *occurrences += 1;

        *occurrences <= maximum_tracks_by_artist
    })
}

/// Sorts tracks for a given section
fn sort_tracks(tracks: &mut [Track], section_type: SectionType) {
    match section_type {
        SectionType::Unplayed => {
            tracks.sort_by_key(|t| (Reverse(t.rating()), t.plays(), t.last_played()))
        }
        SectionType::LeastPlayed => tracks.sort_by_key(|t| (t.plays(), t.last_played())),
        SectionType::Oldest => tracks.sort_by_key(|t| (t.last_played(), t.plays())),
    }
}

/// Randomizes tracks for a given section
fn randomizer(tracks: &mut Vec<Track>, section_type: SectionType) {
    *tracks = tracks
        .iter()
        .fold(
            BTreeMap::new(),
            |mut acc: BTreeMap<String, Vec<Track>>, track| {
                let key = match section_type {
                    SectionType::Oldest => track.last_played_year_and_month(),
                    _ => track.plays().to_string(),
                };
                let value = acc.entry(key).or_default();
                value.push(track.clone());
                acc
            },
        )
        .iter_mut()
        .fold(Vec::new(), |mut acc, (_, group)| {
            group.shuffle(&mut rand::thread_rng());
            acc.append(group);
            acc
        })
}

/// Reduces a list of tracks to a given time limit
fn reduce_to_time_limit(tracks: &mut Vec<Track>, time_limit: f64) {
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
