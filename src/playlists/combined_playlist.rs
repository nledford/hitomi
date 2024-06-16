use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use log::{error, info};
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use rayon::prelude::ParallelSliceMut;

use crate::playlists::{dedup_lists, get_tracks_within_time_range, remove_duplicates, TimeRange};
use crate::plex;
use crate::plex::models::Track;

static SIX_HOURS: Duration = Duration::from_secs(6 * 60 * 60);
static EIGHT_HOURS: Duration = Duration::from_secs(8 * 60 * 60);


pub struct CombinedPlaylist {
    least_tracks: Vec<Track>,
    oldest_tracks: Vec<Track>,
    combined_tracks: Vec<Track>,
}

impl CombinedPlaylist {
    pub async fn build() -> Result<()> {
        let mut cp = Self {
            least_tracks: vec![],
            oldest_tracks: vec![],
            combined_tracks: vec![],
        };

        info!("Building combined playlist...");

        if let Err(err) = cp.fetch_tracks().await {
            error!("An error occurred while attempting to fetch tracks: {err}");
            error!("Skipping building combined playlist.");
            return Ok(());
        }

        cp.dedup_individual_lists();
        cp.dedup_lists();
        cp.sort_final_lists();
        cp.combine_playlists();

        if let Err(err) = cp.update_playlist().await {
            error!("An error occurred while attempting to update combined playlist: {err}");
            error!("Skipping building combined playlist.");
            return Ok(());
        }

        cp.show_results();

        Ok(())
    }

    fn played_tracks(&self) -> Vec<Track> {
        self.least_tracks
            .par_iter()
            .filter_map(|i| if !i.never_played() { Some(i.to_owned()) } else { None })
            .collect::<Vec<Track>>()
    }

    fn unplayed_tracks(&self) -> Vec<Track> {
        self.least_tracks
            .par_iter()
            .filter_map(|i| if i.never_played() { Some(i.to_owned()) } else { None })
            .collect::<Vec<Track>>()
    }

    fn unplayed_tracks_duration(&self) -> Duration {
        self.unplayed_tracks()
            .par_iter()
            .fold(|| Duration::from_secs(0), |acc, x| {
                acc + Duration::from_millis(x.duration() as u64)
            })
            .sum()
    }

    fn unplayed_tracks_is_six_hours(&self) -> bool {
        self.unplayed_tracks_duration() >= SIX_HOURS
    }

    fn unplayed_tracks_is_eight_hours(&self) -> bool {
        if self.unplayed_tracks_is_six_hours() {
            self.unplayed_tracks_duration() >= EIGHT_HOURS
        } else {
            false
        }
    }

    async fn fetch_tracks(&mut self) -> anyhow::Result<()> {
        info!("Fetching tracks...");

        self.fetch_least_played_tracks().await?;
        self.fetch_oldest_tracks().await?;

        Ok(())
    }

    async fn fetch_least_played_tracks(&mut self) -> anyhow::Result<()> {
        let mut filters = HashMap::new();
        filters.insert("userRating>>=", "6");
        filters.insert("lastViewedAt<<=", "-24h");

        let sort = vec![
            "track.viewCount",
            "track.lastViewedAt",
            "guid",
            "mediaBitrate:desc",
        ];

        let tracks = plex::fetch_music(filters, sort, Some(1111)).await?;

        self.least_tracks = tracks;

        Ok(())
    }

    async fn fetch_oldest_tracks(&mut self) -> anyhow::Result<()> {
        let mut filters = HashMap::new();
        filters.insert("userRating>>=", "6");
        filters.insert("viewCount>>=", "0");
        filters.insert("lastViewedAt<<=", "-6w");

        let sort = vec!["lastViewedAt", "viewCount", "guid", "mediaBitrate:desc"];

        let tracks = plex::fetch_music(filters, sort, Some(1111)).await?;

        self.oldest_tracks = tracks;

        Ok(())
    }

    fn dedup_individual_lists(&mut self) {
        info!("De-duplicating individual lists...");

        let least_length = self.least_tracks.len();
        let oldest_length = self.oldest_tracks.len();

        remove_duplicates(&mut self.least_tracks, true, true, true, true);
        remove_duplicates(&mut self.oldest_tracks, true, true, true, false);

        self.log_combined_deduplication(least_length, oldest_length);
    }

    fn dedup_lists(&mut self) {
        info!("De-duplicating lists...");

        let least_length = self.least_tracks.len();
        let oldest_length = self.oldest_tracks.len();

        dedup_lists(&mut self.oldest_tracks, &self.least_tracks);
        dedup_lists(&mut self.least_tracks, &self.oldest_tracks);

        self.log_combined_deduplication(least_length, oldest_length);
    }

    fn sort_final_lists(&mut self) {
        info!("Sorting final lists...");
        self.sort_least_tracks();
        self.sort_oldest_tracks();
    }

    fn combine_playlists(&mut self) {
        let mut rng = fastrand::Rng::new();

        info!("Combining playlists...");

        if self.unplayed_tracks_is_eight_hours() {
            let mut unplayed_tracks = get_tracks_within_time_range(&self.unplayed_tracks(), TimeRange::EightHours);
            rng.shuffle(&mut unplayed_tracks);

            let mut played_tracks = get_tracks_within_time_range(&self.played_tracks(), TimeRange::EightHours);
            rng.shuffle(&mut played_tracks);

            let mut oldest = get_tracks_within_time_range(&self.oldest_tracks, TimeRange::EightHours);
            rng.shuffle(&mut oldest);

            let unplayed_len = unplayed_tracks.len();
            let played_len = played_tracks.len();
            let oldest_len = oldest.len();

            let limit = *[unplayed_len, played_len, oldest_len].iter().max().unwrap();

            for i in 0..limit {
                if i < unplayed_len {
                    self.combined_tracks.push(unplayed_tracks[i].clone())
                }

                if i < played_len {
                    self.combined_tracks.push(played_tracks[i].clone())
                }

                if i < oldest_len {
                    self.combined_tracks.push(oldest[i].clone())
                }
            }

            return;
        }

        let mut least = if self.unplayed_tracks_is_six_hours() {
            let mut unplayed_tracks =
                get_tracks_within_time_range(&self.unplayed_tracks(), TimeRange::SixHours);
            rng.shuffle(&mut unplayed_tracks);

            let mut played_tracks =
                get_tracks_within_time_range(&self.played_tracks(), TimeRange::SixHours);
            rng.shuffle(&mut played_tracks);

            unplayed_tracks.append(&mut played_tracks);
            unplayed_tracks
        } else {
            get_tracks_within_time_range(&self.least_tracks, TimeRange::TwelveHours)
        };
        rng.shuffle(&mut least);

        let mut oldest = get_tracks_within_time_range(&self.oldest_tracks, TimeRange::TwelveHours);
        rng.shuffle(&mut oldest);

        let least_length = least.len();
        let oldest_length = oldest.len();
        let limit = *[least_length, oldest_length].iter().max().unwrap();

        for i in 0..limit {
            if i < least_length {
                self.combined_tracks.push(least[i].clone())
            }

            if i < oldest_length {
                self.combined_tracks.push(oldest[i].clone())
            }
        }
    }

    async fn update_playlist(&self) -> Result<()> {
        let summary = format!("UNPLAYED TRACKS: {}\nA Generated playlist that combines unplayed, least played, and oldest tracks.",
                              self.combined_tracks.par_iter().filter(|t| t.never_played()).count(),
        );

        plex::update_playlist("HOF: Combined", &self.combined_tracks, &summary).await?;

        Ok(())
    }

    fn show_results(&self) {
        plex::show_results(&self.combined_tracks)
    }

    fn log_combined_deduplication(
        &self,
        original_least_length: usize,
        original_oldest_length: usize,
    ) {
        info!(
            "\tLEAST  - Original Size: {original_least_length} | Deduplicated Size: {}",
            self.least_tracks.len()
        );
        info!(
            "\tOLDEST - Original Size: {original_oldest_length} | Deduplicated Size: {}",
            self.oldest_tracks.len()
        );
    }
}
