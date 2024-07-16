use derive_builder::Builder;
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

    pub fn run_manual_filters(&mut self, profile_section: &ProfileSection, time_limit: f64) {
        if profile_section.is_unplayed_section() {
            self.unplayed = profile_section.run_manual_filters(
                &self.unplayed,
                SectionType::Unplayed,
                time_limit,
            )
        }

        if profile_section.is_least_played_section() {
            self.least_played = profile_section.run_manual_filters(
                &self.least_played,
                SectionType::LeastPlayed,
                time_limit,
            )
        }

        if profile_section.is_oldest_section() {
            self.oldest =
                profile_section.run_manual_filters(&self.oldest, SectionType::Oldest, time_limit)
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

fn deduplicate_tracks_by_lists(tracks: &mut Vec<Track>, lists: Vec<&[Track]>) {
    for list in lists {
        tracks.retain(|t| !list.contains(t))
    }
}
