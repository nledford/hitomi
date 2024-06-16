use std::collections::HashMap;

use anyhow::Result;
use log::{error, info};

use crate::{playlists, plex};
use crate::playlists::{get_tracks_within_time_range, TimeRange};
use crate::plex::models::Track;

/// Represents a playlist of HOF tracks that is entirely random.
pub struct RandomPlaylist {
    tracks: Vec<Track>,
}

impl RandomPlaylist {
    /// Build the random playlist
    pub async fn build() -> Result<()> {
        let mut rp = Self { tracks: vec![] };

        info!("Building random playlist...");

        if let Err(err) = rp.fetch_tracks().await {
            error!("An error occurred while attempting to fetch tracks: {err}");
            error!("Skipping building random playlist.");
            return Ok(());
        }

        rp.dedup_tracks();

        if let Err(err) = rp.update_playlist().await {
            error!("An error occurred while attempt to update playlist: {err}");
            error!("Skipping building random playlist");
            return Ok(());
        }

        rp.show_results();

        Ok(())
    }

    /// Fetches tracks from the plex server
    async fn fetch_tracks(&mut self) -> Result<()> {
        info!("Fetching all HOF tracks...");

        let mut filters = HashMap::new();
        filters.insert("userRating>>=", "6");
        filters.insert("viewCount>>=", "0");

        let sort = vec!["guid", "mediaBitrate:desc"];

        self.tracks = plex::fetch_music(filters, sort, None).await?;

        info!("{} tracks fetched from plex server", self.tracks.len());

        Ok(())
    }

    /// De-duplicates all tracks by the track `guid`
    fn dedup_tracks(&mut self) {
        info!("De-duplicating tracks...");

        playlists::remove_duplicates(&mut self.tracks, true, true, false, false);
    }

    /// Update the playlist on the plex server
    async fn update_playlist(&mut self) -> Result<()> {
        let mut rng = fastrand::Rng::new();

        // Randomize tracks
        rng.shuffle(&mut self.tracks);
        self.tracks = get_tracks_within_time_range(&self.tracks, TimeRange::TwentyFourHours);

        let summary = "A generated playlist of all HOF tracks that is totally random.";

        plex::update_playlist("HOF: Random", &self.tracks, summary).await?;

        Ok(())
    }

    /// Display results of the update
    fn show_results(&self) {
        plex::show_results(&self.tracks)
    }
}
