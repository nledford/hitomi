#![allow(dead_code)]

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap};

use anyhow::Result;
use chrono::{Duration, TimeDelta};
use derive_builder::Builder;
use itertools::Itertools;
use rand::prelude::SliceRandom;
use simplelog::info;

use crate::db;
use crate::plex::models::tracks::Track;
use crate::plex::PlexClient;
use crate::profiles::profile::Profile;
use crate::profiles::profile_section::ProfileSection;
use crate::profiles::{ProfileSource, SectionType};

#[derive(Builder, Clone)]
pub struct ProfileTracks {
    #[builder(default)]
    unplayed: Vec<Track>,
    #[builder(default)]
    least_played: Vec<Track>,
    #[builder(default)]
    oldest: Vec<Track>,
    #[builder(default)]
    merged: Vec<Track>,
}

impl ProfileTracks {
    pub async fn new(plex_client: &PlexClient, profile: &Profile) -> Result<Self> {
        let profile_tracks = fetch_profile_tracks(plex_client, profile).await?;
        Ok(profile_tracks)
    }

    pub fn have_unplayed_tracks(&self) -> bool {
        !self.unplayed.is_empty()
    }

    pub fn have_least_played_tracks(&self) -> bool {
        !self.least_played.is_empty()
    }

    pub fn have_oldest_tracks(&self) -> bool {
        !self.oldest.is_empty()
    }

    fn get_section_tracks(&self, section_type: SectionType) -> &[Track] {
        match section_type {
            SectionType::Unplayed => &self.unplayed,
            SectionType::LeastPlayed => &self.least_played,
            SectionType::Oldest => &self.oldest,
        }
    }

    fn get_section_tracks_mut(&mut self, section_type: SectionType) -> &mut Vec<Track> {
        match section_type {
            SectionType::Unplayed => &mut self.unplayed,
            SectionType::LeastPlayed => &mut self.least_played,
            SectionType::Oldest => &mut self.oldest,
        }
    }

    fn get_num_tracks_by_section(&self, section_type: SectionType) -> usize {
        self.get_section_tracks(section_type).len()
    }

    fn get_total_duration_of_section(&self, section_type: SectionType) -> Duration {
        let tracks = self.get_section_tracks(section_type);
        let total = tracks.iter().fold(TimeDelta::seconds(0), |mut acc, track| {
            acc += track.get_track_duration_timedelta();
            acc
        });
        Duration::from(total)
    }

    /// Returns a slice of the merged tracks
    pub fn get_merged_tracks(&self) -> &[Track] {
        &self.merged
    }

    /// Returns `false` if no sections are valid
    fn get_none_are_valid(&self) -> bool {
        self.get_num_valid() == 0
    }

    /// Returns the number of valid sections (those that are not empty)
    fn get_num_valid(&self) -> usize {
        [
            self.have_unplayed_tracks(),
            self.have_least_played_tracks(),
            self.have_oldest_tracks(),
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
                .map(|track| track.get_id().to_string())
                .collect::<Vec<_>>()
        }
    }
}

/*** Filters ***/
impl ProfileTracks {
    /// Runs manual filters for the profile sections
    ///
    /// Manual filters are those that are unique to this application and not included with plex
    pub fn run_manual_filters(&mut self, profile_sections: &[ProfileSection], time_limit: f64) {
        info!("Running manual section filters...");

        for section in profile_sections {
            let tracks = self.get_section_tracks_mut(section.get_section_type());
            remove_played_within_last_day(tracks);

            if section.get_deduplicate_tracks_by_guid() {
                deduplicate_by_track_guid(tracks);
            }
        }

        self.deduplicate_lists(time_limit);

        for section in profile_sections {
            let tracks = self.get_section_tracks_mut(section.get_section_type());

            if section.get_deduplicate_tracks_by_title_and_artist() {
                deduplicate_by_title_and_artist(tracks);
            }

            trim_tracks_by_artist(
                tracks,
                section.get_maximum_tracks_by_artist(),
                section.get_section_type(),
            );

            sort_tracks(tracks, section.get_section_type());

            if time_limit > 0.0 {
                reduce_to_time_limit(tracks, time_limit);
            }

            if section.get_randomize_tracks() {
                randomizer(tracks, section.get_section_type())
            }
        }
    }

    /// Deduplicates the least played and oldest tracks
    ///
    /// Least played is deduplicated first, and oldest is deduplicated second
    fn deduplicate_lists(&mut self, time_limit: f64) {
        if !self.have_oldest_tracks() || !self.have_least_played_tracks() {
            return;
        }

        if time_limit <= 0.0 {
            panic!("Time limit cannot be less than or equal to zero")
        }

        deduplicate_tracks_by_lists(&mut self.least_played, &self.oldest, time_limit);
        // deduplicate_tracks_by_lists(&mut self.oldest, &self.least_played, time_limit);
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
        if self.get_none_are_valid() {
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
}

/// Deduplicates one list based on values in other lists
fn deduplicate_tracks_by_lists(tracks: &mut Vec<Track>, comp: &[Track], time_limit: f64) {
    loop {
        let orig_len = tracks.len();

        let mut tracks_chunks = chunk_by_time_limit(tracks, time_limit);
        let comp_chunks = chunk_by_time_limit(comp, time_limit);

        for i in 0..(tracks_chunks.len() as i32) {
            let track_chunk = tracks_chunks.get_mut(&i);
            let comp_chunk = comp_chunks.get(&i);
            if let (Some(track_chunk), Some(comp_chunk)) = (track_chunk, comp_chunk) {
                track_chunk.retain(|track| !comp_chunk.contains(track));
            }
        }

        *tracks = tracks_chunks
            .into_values()
            .fold(Vec::new(), |mut acc, mut tracks| {
                acc.append(&mut tracks);
                acc
            });

        let new_len = tracks.len();
        if new_len == orig_len {
            break;
        }
    }
}

/// Remove duplicate tracks by the title and artist of a track
///
/// e,g, If the track "The Beatles - Get Back" appears multiple times in a playlist, any duplicates will be removed.
fn deduplicate_by_title_and_artist(tracks: &mut Vec<Track>) {
    *tracks = tracks
        .iter()
        .sorted_by_key(|track| track.get_title_and_artist_sort_key())
        .unique_by(|track| track.get_title_and_artist_sort_key())
        .map(|track| track.to_owned())
        .collect_vec()
}

/// Remove duplicate tracks based on the Plex `GUID`
fn deduplicate_by_track_guid(tracks: &mut Vec<Track>) {
    *tracks = tracks
        .iter()
        .sorted_by_key(|track| (track.get_guid(), Reverse(track.get_bitrate())))
        .unique_by(|track| track.get_guid())
        .map(|track| track.to_owned())
        .collect_vec()
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
        SectionType::Oldest => {
            tracks.sort_by_key(|track| (track.get_last_played(), track.get_plays()))
        }
        _ => tracks.sort_by_key(|track| (track.get_plays(), track.get_last_played())),
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
            tracks.sort_by_key(|t| (Reverse(t.get_rating()), t.get_plays(), t.get_last_played()))
        }
        SectionType::LeastPlayed => tracks.sort_by_key(|t| (t.get_plays(), t.get_last_played())),
        SectionType::Oldest => tracks.sort_by_key(|t| (t.get_last_played(), t.get_plays())),
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
                    SectionType::Oldest => track.get_last_played_year_and_month(),
                    _ => format!(
                        "{:04}: {}",
                        track.get_plays(),
                        track.get_last_played_year_and_month(),
                    ),
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
    let index = determine_time_limit_index(tracks, time_limit);
    *tracks = tracks
        .iter()
        .get(0..=index)
        .map(|x| x.to_owned())
        .collect_vec();
}

fn determine_time_limit_index(tracks: &[Track], time_limit: f64) -> usize {
    if time_limit == 0.0 {
        return tracks.len();
    }

    let limit = TimeDelta::seconds((time_limit * 60_f64 * 60_f64) as i64);

    let total_duration: i64 = tracks.iter().map(|track| track.get_track_duration()).sum();
    let total_duration = TimeDelta::milliseconds(total_duration);

    if total_duration <= limit {
        return tracks.len();
    }

    let mut accum_total = TimeDelta::seconds(0);
    let index = tracks
        .iter()
        .position(|track| {
            accum_total += track.get_track_duration_timedelta();
            accum_total > limit
        })
        .unwrap_or(0);

    index
}

/// Splits a list of tracks into chunks by a given time limit
///
/// # Example
///
/// If the list of tracks is 72 hours long and the playlist time limit is 12 hours,
/// then 6 chunks will be returned.
fn chunk_by_time_limit(tracks: &[Track], time_limit: f64) -> BTreeMap<i32, Vec<Track>> {
    let mut remaining_tracks = tracks.to_vec();
    let mut chunks: BTreeMap<i32, Vec<Track>> = BTreeMap::new();

    let mut day = 1;
    let mut index;

    while !remaining_tracks.is_empty() {
        index = determine_time_limit_index(&remaining_tracks, time_limit);
        let mut day_tracks = remaining_tracks.drain(..index).collect_vec();

        let entry = chunks.entry(day).or_default();
        entry.append(&mut day_tracks);

        day += 1;
    }

    chunks
}

fn remove_played_within_last_day(tracks: &mut Vec<Track>) {
    *tracks = tracks
        .iter()
        .filter_map(|track| {
            if !track.get_played_within_last_day() {
                Some(track.to_owned())
            } else {
                None
            }
        })
        .collect_vec()
}

async fn fetch_profile_tracks(
    plex_client: &PlexClient,
    profile: &Profile,
) -> Result<ProfileTracks> {
    let sections =
        db::profiles::fetch_profile_sections_for_profile(profile.get_profile_id()).await?;

    let mut profile_tracks = ProfileTracksBuilder::default();
    for section in &sections {
        let tracks = fetch_section_tracks(
            plex_client,
            profile,
            section,
            profile.get_time_limit() as f64,
        )
            .await?;

        match section.get_section_type() {
            SectionType::Unplayed => {
                profile_tracks.unplayed(tracks);
            }
            SectionType::LeastPlayed => {
                profile_tracks.least_played(tracks);
            }
            SectionType::Oldest => {
                profile_tracks.oldest(tracks);
            }
        }
    }
    let mut profile_tracks = profile_tracks
        .build()
        .expect("Profile tracks could not be built");
    profile_tracks.run_manual_filters(&sections, profile.get_section_time_limit());
    profile_tracks.merge();

    Ok(profile_tracks)
}

async fn fetch_section_tracks(
    plex_client: &PlexClient,
    profile: &Profile,
    section: &ProfileSection,
    time_limit: f64,
) -> Result<Vec<Track>> {
    let mut tracks = vec![];

    if !section.is_enabled() {
        return Ok(tracks);
    }
    let mut filters = HashMap::new();
    if section.get_minimum_track_rating_adjusted() != 0 {
        filters.insert(
            "userRating>>".to_string(),
            section.get_minimum_track_rating_adjusted().to_string(),
        );
    }

    if section.is_unplayed_section() {
        filters.insert("viewCount".to_string(), "0".to_string());
    } else {
        filters.insert("viewCount>>".to_string(), "0".to_string());
    }

    match profile.get_profile_source() {
        // Nothing special needs to be done for a library source, so this branch is left blank
        ProfileSource::Library => {}
        ProfileSource::Collection => {
            let collection = plex_client
                .fetch_collection(profile.get_profile_source_id().unwrap())
                .await?;
            let artists = plex_client
                .fetch_artists_from_collection(&collection)
                .await?;
            let artists = artists.join(",");
            filters.insert("artist.id".to_string(), artists);
        }
        ProfileSource::SingleArtist => {
            filters.insert(
                "artist.id".to_string(),
                profile.get_profile_source_id().unwrap().to_string(),
            );
        }
    }

    let limit = if time_limit <= 0.0 {
        None
    } else {
        Some((400.0 * (time_limit / 12.0)).floor() as i32)
    };
    tracks = plex_client
        .fetch_music(filters, section.get_sorting_vec(), limit)
        .await?;

    Ok(tracks)
}
