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

    pub fn run_manual_filters(&mut self, profile_sections: &[ProfileSection], time_limit: f64) {
        info!("Running manual section filters...");

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

    pub fn deduplicate_lists(&mut self) {
        deduplicate_tracks_by_lists(&mut self.least_played, vec![&self.oldest]);
        deduplicate_tracks_by_lists(&mut self.oldest, vec![&self.least_played]);
    }

    pub fn get_combined_tracks(&self) -> &[Track] {
        &self.merged
    }

    fn none_are_valid(&self) -> bool {
        self.get_num_valid() == 0
    }

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

    pub fn print_preview(&self) {
        if self.merged.is_empty() {
            return;
        }

        let preview = self.merged.iter().take(25).collect::<Vec<_>>();

        for (i, track) in preview.iter().enumerate() {
            println!("{:2} {}", i + 1, track)
        }
    }

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

fn deduplicate_by_title_and_artist(tracks: &mut Vec<Track>) {
    tracks.sort_by_key(|track| (track.title().to_owned(), track.artist().to_owned()));
    tracks.dedup_by_key(|track| (track.title().to_owned(), track.artist().to_owned()));
}

fn deduplicate_by_track_guid(tracks: &mut Vec<Track>) {
    tracks.dedup_by_key(|track| track.get_guid().to_owned());
}

fn deduplicate_tracks_by_lists(tracks: &mut Vec<Track>, lists: Vec<&[Track]>) {
    for list in lists {
        tracks.retain(|t| !list.contains(t))
    }
}

fn trim_tracks_by_artist(
    tracks: &mut Vec<Track>,
    maximum_tracks_by_artist: u32,
    section_type: SectionType,
) {
    if maximum_tracks_by_artist == 0 {
        return;
    }
    info!("Trimming tracks by artists...");

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

fn sort_tracks(tracks: &mut [Track], section_type: SectionType) {
    info!("Sorting section tracks...");

    match section_type {
        SectionType::Unplayed => {
            tracks.sort_by_key(|t| (Reverse(t.rating()), t.plays(), t.last_played()))
        }
        SectionType::LeastPlayed => tracks.sort_by_key(|t| (t.plays(), t.last_played())),
        SectionType::Oldest => tracks.sort_by_key(|t| (t.last_played(), t.plays())),
    }
}

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

fn reduce_to_time_limit(tracks: &mut Vec<Track>, time_limit: f64) {
    info!("Trimming section tracks to time limit...");

    let limit = TimeDelta::seconds((time_limit * 60_f64 * 60_f64) as i64);

    let total_duration: i64 = tracks.iter().map(|track| track.duration()).sum();
    let total_duration = TimeDelta::milliseconds(total_duration);

    if total_duration <= limit {
        info!("Section tracks do not meet or exceed time limit. Skipping...");
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
