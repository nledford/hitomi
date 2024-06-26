use serde::{Deserialize, Serialize};
use strum::EnumString;

use crate::plex::types::PlaylistTitle;

#[derive(Clone, Default, Debug, Deserialize, EnumString, Serialize)]
pub enum PlaylistType {
    #[default]
    Audio,
    Photo,
    Video,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    rating_key: String,
    pub key: String,
    pub guid: String,
    // #[serde(alias = "type")]
    // pub item_type: String,
    pub title: PlaylistTitle,
    pub summary: String,
    pub smart: bool,
    pub playlist_type: PlaylistType,
    // pub composite: String,
    // pub icon: Option<String>,
    // pub view_count: i32,
    // pub last_viewed_at: u128,
    pub duration: Option<u128>,
    pub leaf_count: i32,
    // pub added_at: u128,
    // pub updated_at: u128,
}

impl Playlist {
    pub fn get_id(&self) -> &str {
        &self.rating_key
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }
}
