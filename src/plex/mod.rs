use std::collections::HashMap;

use anyhow::{anyhow, Result};
use default_struct_builder::DefaultBuilder;
use log::{error, info};
use rayon::prelude::*;
use serde::Deserialize;
use simplelog::debug;

use crate::config::Config;
use crate::http_client::HttpClient;
use crate::plex::models::{
    Artist, Collection, MediaContainerWrapper, NewPlaylist, Playlist, PlexResponse, Section,
    SectionResponse, Track,
};
use crate::profiles::profile::Profile;

pub mod models;

#[derive(Debug, Default, DefaultBuilder)]
pub struct Plex {
    client: HttpClient,
    plex_token: String,
    plex_url: String,
    machine_identifier: String,

    primary_section_id: i32,

    playlists: Vec<Playlist>,
    collections: Vec<Collection>,
    sections: Vec<Section>,
}

impl Plex {
    pub async fn initialize(config: &Config) -> Result<Self> {
        debug!("Initializing plex...");

        if !config.is_loaded() {
            return Err(anyhow!(
                "Cannot initialize plex because application config has not yet been loaded."
            ));
        }

        let plex_url = config.get_plex_url();
        let plex_token = config.get_plex_token();

        let client = HttpClient::new(plex_url, plex_token)?;

        let mut plex = Self::default()
            .client(client)
            .plex_token(plex_token.to_string())
            .plex_url(plex_url.to_string())
            .primary_section_id(config.get_primary_section_id());

        plex.get_machine_identifier().await?;
        plex.fetch_music_sections().await?;
        plex.fetch_collections().await?;
        plex.fetch_playlists().await?;

        Ok(plex)
    }

    pub async fn new_for_config(plex_url: &str, plex_token: &str) -> Result<Self> {
        let client = HttpClient::new(plex_url, plex_token)?;

        let mut plex = Self::default()
            .client(client)
            .plex_token(plex_token.to_string())
            .plex_url(plex_url.to_string());

        plex.fetch_music_sections().await?;

        Ok(plex)
    }

    async fn fetch_collections(&mut self) -> Result<()> {
        let resp: PlexResponse<Vec<Collection>> = self
            .client
            .get(
                &format!("library/sections/{}/collections", self.primary_section_id),
                None,
                None,
            )
            .await?;

        self.collections = resp.media_container.metadata;

        Ok(())
    }

    pub fn get_collections(&self) -> Vec<Collection> {
        self.collections.clone()
    }

    pub async fn fetch_music_sections(&mut self) -> Result<()> {
        let resp: SectionResponse = self.client.get("library/sections", None, None).await?;

        let sections = resp.media_container.directory;
        self.sections = sections
            .into_iter()
            .filter(|s| s.is_type_music())
            .collect::<_>();

        Ok(())
    }

    pub fn get_music_sections(&self) -> &[Section] {
        &self.sections
    }

    async fn fetch_playlists(&mut self) -> Result<()> {
        let resp: PlexResponse<Vec<Playlist>> = self.client.get("playlists", None, None).await?;

        self.playlists = resp.media_container.metadata;
        Ok(())
    }

    pub fn get_playlists(&self) -> Vec<Playlist> {
        self.playlists.clone()
    }

    pub fn get_playlist(&self, playlist_id: &str) -> &Playlist {
        self.playlists
            .iter()
            .find(|p| p.rating_key == playlist_id)
            .unwrap()
    }

    pub async fn fetch_playlist_items(&self, playlist_id: &str) -> Result<Vec<Track>> {
        let resp: PlexResponse<Vec<Track>> = self
            .client
            .get(&format!("playlists/{playlist_id}/items"), None, None)
            .await?;
        Ok(resp.media_container.metadata)
    }

    pub async fn fetch_music(
        &self,
        filters: HashMap<String, String>,
        sort: Vec<&str>,
        max_results: Option<i32>,
    ) -> Result<Vec<Track>> {
        let sort = &sort.join(",");

        let mut params = HashMap::new();
        params.insert("type".to_string(), "10".to_string());
        params.insert("sort".to_string(), sort.to_string());
        params.extend(filters);

        let resp: Result<PlexResponse<Vec<Track>>> = self
            .client
            .get("library/sections/5/all", Some(params), max_results)
            .await;

        match resp {
            Ok(resp) => Ok(resp.media_container.metadata),
            Err(err) => {
                error!("An error occurred while attempting to fetch tracks: {err}");
                Err(err)
            }
        }
    }

    pub async fn update_playlist(
        &self,
        playlist_id: &str,
        tracks: &[Track],
        summary: &str,
    ) -> Result<()> {
        info!("Wiping destination playlist...");
        self.clear_playlist(playlist_id).await?;

        info!("Updating destination playlist...");
        let ids = tracks.par_iter().map(|t| t.id()).collect::<Vec<&str>>();
        for chunk in ids.chunks(200) {
            self.add_items_to_playlist(playlist_id, chunk).await?;
        }

        // TODO add last updated/next update to summary
        self.update_summary(playlist_id, summary).await?;

        Ok(())
    }

    pub async fn update_summary(&self, playlist_id: &str, summary: &str) -> Result<()> {
        let params = HashMap::from([(
            "summary".to_string(),
            urlencoding::encode(summary).to_string(),
        )]);

        let _ = self
            .client
            .put(&format!("playlists/{}", playlist_id), Some(params))
            .await?;

        Ok(())
    }

    pub async fn create_playlist(&self, profile: &Profile) -> Result<String> {
        let params = HashMap::from([
            (
                "uri".to_string(),
                format!(
                    "{}/library/metadata",
                    self.uri_root(),
                    // items.iter()
                    //     .map(|x| x.id())
                    //     .collect::<Vec<&str>>()
                    //     .join(",")
                ),
            ),
            (
                "title".to_string(),
                urlencoding::encode(profile.get_title()).to_string(),
            ),
            // ("summary".to_string(), urlencoding::encode(profile.get_summary()).to_string()),
            ("smart".to_string(), "0".to_string()),
            ("type".to_string(), "audio".to_string()),
        ]);

        let playlist: PlexResponse<Vec<NewPlaylist>> =
            self.client.post("playlists", Some(params)).await?;
        let playlist = playlist.media_container.metadata.get(0).unwrap();

        Ok(playlist.rating_key.to_string())
    }

    pub async fn add_items_to_playlist(&self, playlist_id: &str, items: &[&str]) -> Result<()> {
        if playlist_id.is_empty() {
            return Err(anyhow!("`playlist_id` is blank"));
        }

        if items.is_empty() {
            return Err(anyhow!("There are no items to add to the playlist"));
        }

        for chunk in items.chunks(200) {
            let params = HashMap::from([(
                "uri".to_string(),
                format!("{}/library/metadata/{}", self.uri_root(), chunk.join(",")),
            )]);

            let _: PlexResponse<Vec<NewPlaylist>> = self
                .client
                .put(&format!("playlists/{playlist_id}/items"), Some(params))
                .await?;
        }

        Ok(())
    }

    pub async fn fetch_artists_from_collection(&self, collection_id: &str) -> Result<Vec<String>> {
        let resp: PlexResponse<Vec<Collection>> = self
            .client
            .get(
                &format!(
                    "library/sections/{}/collection/{collection_id}",
                    self.primary_section_id
                ),
                None,
                None,
            )
            .await?;
        let artists = resp
            .media_container
            .metadata
            .into_iter()
            .map(|item| item.rating_key)
            .collect::<_>();

        Ok(artists)
    }

    pub async fn search_for_artist(&self, artist: &str) -> Result<Vec<Artist>> {
        let params = HashMap::from([("title".to_string(), artist.to_string())]);

        let resp: PlexResponse<Vec<Artist>> = self
            .client
            .get(
                &format!("/library/sections/{}/all", self.primary_section_id),
                Some(params),
                Some(10),
            )
            .await?;

        Ok(resp.media_container.metadata)
    }

    pub async fn clear_playlist(&self, playlist_id: &str) -> Result<()> {
        self.client
            .delete(&format!("playlists/{playlist_id}/items"), None)
            .await?;
        Ok(())
    }

    async fn get_machine_identifier(&mut self) -> Result<()> {
        #[derive(Deserialize)]
        struct Identity {
            #[serde(alias = "machineIdentifier")]
            machine_identifier: String,
        }

        let resp: MediaContainerWrapper<Identity> = self.client.get("identity", None, None).await?;
        self.machine_identifier = resp.media_container.machine_identifier;

        Ok(())
    }

    fn uri_root(&self) -> String {
        format!(
            "server://{}/com.plexapp.plugins.library",
            &self.machine_identifier
        )
    }
}
