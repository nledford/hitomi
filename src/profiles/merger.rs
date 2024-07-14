use simplelog::info;
use crate::plex::models::tracks::Track;

#[derive(Debug, Default)]
pub struct SectionTracksMerger {
    unplayed: Vec<Track>,
    least_played: Vec<Track>,
    oldest: Vec<Track>,
    combined: Vec<Track>,
}

impl SectionTracksMerger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_unplayed_tracks(&self) -> &[Track] {
        &self.unplayed
    }

    pub fn set_unplayed_tracks(&mut self, tracks: Vec<Track>) {
        self.unplayed = tracks
    }

    pub fn get_least_played_tracks(&self) -> &[Track] {
        &self.least_played
    }

    pub fn set_least_played_tracks(&mut self, tracks: Vec<Track>) {
        self.least_played = tracks
    }

    pub fn get_oldest_tracks(&self) -> &[Track] {
        &self.oldest
    }

    pub fn set_oldest_tracks(&mut self, tracks: Vec<Track>) {
        self.oldest = tracks
    }

    pub fn get_combined_tracks(&self) -> &[Track] {
        &self.combined
    }

    fn are_none_valid(&self) -> bool {
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
        if self.combined.is_empty() {
            vec![]
        } else {
            self.combined
                .iter()
                .map(|track| track.id().to_string())
                .collect::<Vec<_>>()
        }
    }
    pub fn print_preview(&self) {
        if self.combined.is_empty() {
            return;
        }

        let preview = self.combined.iter().take(25).collect::<Vec<_>>();

        for (i, track) in preview.iter().enumerate() {
            println!("{:2} {}", i + 1, track)
        }
    }

    pub fn merge(&mut self) {
        if self.are_none_valid() {
            return;
        }
        info!("Combining playlists...");

        self.combined = Vec::new();

        info!("Combing {} sections...", self.get_num_valid());

        for i in 0..self.get_largest_section_length() {
            if let Some(track) = self.unplayed.get(i) {
                self.combined.push(track.clone())
            }

            if let Some(track) = self.least_played.get(i) {
                self.combined.push(track.clone())
            }

            if let Some(track) = self.oldest.get(i) {
                self.combined.push(track.clone())
            }
        }
    }
}