use std::collections::HashMap;

use anyhow::{anyhow, Result};
use derive_builder::Builder;
use log::{error, info};
use serde::Deserialize;
use simplelog::debug;

use crate::config::Config;
use crate::http_client::HttpClient;
use crate::plex::models::artists::Artist;
use crate::plex::models::collections::Collection;
use crate::plex::models::new_playlist::NewPlaylist;
use crate::plex::models::playlists::Playlist;
use crate::plex::models::sections::Section;
use crate::plex::models::tracks::Track;
use crate::plex::models::{MediaContainerWrapper, PlexResponse, SectionResponse};
use crate::plex::types::{PlaylistId, PlexToken, PlexUrl};
use crate::profiles::profile::Profile;

pub mod models;
pub mod types;

/// Plex API wrapper
///
/// Dead code is allowed for this specific struct due to [`DefaultBuilder`](default_struct_builder::DefaultBuilder)
/// using both the `plex_token` and `plex_url` fields.
#[allow(dead_code)]
#[derive(Builder, Clone, Debug, Default)]
pub struct PlexClient {
    client: HttpClient,
    plex_token: PlexToken,
    plex_url: PlexUrl,
    #[builder(default)]
    machine_identifier: String,

    primary_section_id: i32,

    #[builder(default)]
    playlists: Vec<Playlist>,
    #[builder(default)]
    collections: Vec<Collection>,
    #[builder(default)]
    sections: Vec<Section>,
}

impl PlexClient {
    pub async fn initialize(config: &Config) -> Result<Self> {
        debug!("Initializing plex...");

        if !config.is_loaded() {
            return Err(anyhow!(
                "Cannot initialize plex because application config has not yet been loaded."
            ));
        }

        let plex_url = PlexUrl::new(config.get_plex_url())?;
        let plex_token = PlexToken::new(config.get_plex_token())?;

        let client = HttpClient::new(plex_url.as_str(), plex_token.as_str())?;

        let mut plex = PlexClientBuilder::default()
            .client(client)
            .plex_token(plex_token)
            .plex_url(plex_url)
            .primary_section_id(config.get_primary_section_id())
            .build()?;

        plex.fetch_machine_identifier().await?;
        plex.fetch_music_sections().await?;
        plex.fetch_collections().await?;
        plex.fetch_playlists().await?;

        Ok(plex)
    }

    pub async fn new_for_config(plex_url: &PlexUrl, plex_token: &PlexToken) -> Result<Self> {
        let client = HttpClient::new(plex_url.as_str(), plex_token.as_str())?;

        let mut plex = PlexClientBuilder::default()
            .client(client)
            .plex_token(plex_token.to_owned())
            .plex_url(plex_url.to_owned())
            .build()?;

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

    pub fn get_playlists(&self) -> &[Playlist] {
        &self.playlists
    }

    pub fn get_playlist(&self, playlist_id: &PlaylistId) -> &Playlist {
        self.playlists
            .iter()
            .find(|p| p.get_id() == playlist_id.as_str())
            .unwrap()
    }

    pub async fn fetch_playlist_items(&self, playlist_id: &PlaylistId) -> Result<Vec<Track>> {
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
        let max_results = Some(max_results.unwrap_or(1111));

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
                error!("An error occurred while attempting to fetch tracks:\n{err}");
                Err(err)
            }
        }
    }

    pub async fn update_playlist(
        &self,
        playlist_id: &PlaylistId,
        tracks: &[Track],
        summary: &str,
    ) -> Result<()> {
        info!("Wiping destination playlist...");
        self.clear_playlist(playlist_id).await?;

        info!("Updating destination playlist...");
        let ids = tracks.iter().map(|t| t.id()).collect::<Vec<&str>>();
        for chunk in ids.chunks(200) {
            self.add_items_to_playlist(playlist_id, chunk).await?;
        }

        self.update_summary(playlist_id, summary).await?;

        Ok(())
    }

    pub async fn update_summary(&self, playlist_id: &PlaylistId, summary: &str) -> Result<()> {
        let params = HashMap::from([("summary".to_string(), summary.to_string())]);

        let _: () = self
            .client
            .put(&format!("playlists/{}", playlist_id), Some(params))
            .await?;

        Ok(())
    }

    pub async fn create_playlist(&self, profile: &Profile) -> Result<String> {
        let params = HashMap::from([
            (
                "uri".to_string(),
                format!("{}/library/metadata", self.uri_root(),),
            ),
            ("title".to_string(), profile.get_title().to_string()),
            // ("summary".to_string(), urlencoding::encode(profile.get_summary()).to_string()),
            ("smart".to_string(), "0".to_string()),
            ("type".to_string(), "audio".to_string()),
        ]);

        let playlist: PlexResponse<Vec<NewPlaylist>> =
            self.client.post("playlists", Some(params)).await?;
        let playlist = playlist.media_container.metadata.first().unwrap();

        Ok(playlist.rating_key.to_string())
    }

    pub async fn add_items_to_playlist(
        &self,
        playlist_id: &PlaylistId,
        items: &[&str],
    ) -> Result<()> {
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
                &format!("library/collections/{collection_id}/children"),
                None,
                None,
            )
            .await?;
        let artists = resp
            .media_container
            .metadata
            .into_iter()
            .map(|item| item.get_id().to_owned())
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

    pub async fn clear_playlist(&self, playlist_id: &PlaylistId) -> Result<()> {
        self.client
            .delete(&format!("playlists/{playlist_id}/items"), None)
            .await?;
        Ok(())
    }

    async fn fetch_machine_identifier(&mut self) -> Result<()> {
        debug!("Fetching machine identifier...");

        #[derive(Default, Deserialize)]
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
