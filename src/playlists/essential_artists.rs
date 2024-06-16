use std::collections::HashMap;

use anyhow::Result;
use log::{error, info};

use crate::playlists::{get_tracks_within_time_range, remove_duplicates, TimeRange};
use crate::plex;
use crate::plex::models::Track;

pub struct EssentialArtistsPlaylist {
    essential_artists: Vec<String>,
    tracks: Vec<Track>,
}

impl EssentialArtistsPlaylist {
    pub async fn build() -> Result<()> {
        let mut eap = Self {
            essential_artists: vec![],
            tracks: vec![],
        };

        info!("Building essential artists playlist...");

        if let Err(err) = eap.load_essential_artists().await {
            error!("An error occurred while attempting to load essential artists: {err}");
            error!("Skipping building essential artists playlist.");
            return Ok(());
        }

        if let Err(err) = eap.fetch_tracks().await {
            error!("An error occurred while attempting to fetch tracks: {err}");
            error!("Skipping building essential artists playlist.");
            return Ok(());
        }

        eap.dedup_tracks();

        if let Err(err) = eap.update_playlist().await {
            error!("An error occurred while attempting to update playlist: {err}");
            error!("Skipping building essential artists playlist.");
            return Ok(());
        }

        eap.show_results();

        Ok(())
    }

    async fn load_essential_artists(&mut self) -> Result<()> {
        info!("Loading essential artists from text file...");
        self.essential_artists = plex::get_essential_artists().await?;
        info!("{} artists loaded", self.essential_artists.len());

        Ok(())
    }

    async fn fetch_tracks(&mut self) -> Result<()> {
        info!("Fetching tracks...");

        let artists = self.essential_artists.join(",");
        let artists = urlencoding::encode(&artists);

        let mut filters = HashMap::new();
        filters.insert("track.userRating>>=", "6");
        filters.insert("artist.id=", &artists);

        let sort = vec![
            "track.lastViewedAt",
            "track.viewCount",
            "track.guid",
            "mediaBitrate:desc",
        ];

        self.tracks = plex::fetch_music(filters, sort, None).await?;

        info!("{} tracks fetched from plex server", self.tracks.len());

        Ok(())
    }

    fn dedup_tracks(&mut self) {
        info!("De-duplicating playlists...");
        remove_duplicates(&mut self.tracks, true, false, false, false)
    }

    async fn update_playlist(&mut self) -> Result<()> {
        let summary = "A generated playlist of HOF tracks by essential artists.";
        self.tracks = get_tracks_within_time_range(&self.tracks, TimeRange::TwentyFourHours);

        let mut rng = fastrand::Rng::new();
        rng.shuffle(&mut self.tracks);

        plex::update_playlist("HOF: Essential Artists", &self.tracks, summary).await?;

        Ok(())
    }

    fn show_results(&self) {
        plex::show_results(&self.tracks);
    }
}
