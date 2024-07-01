use crate::plex::types::PlexId;
use crate::types::Title;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    rating_key: PlexId,
    title: Title,
    summary: String,
    // smart: bool,
    duration: Option<u128>,
    leaf_count: u32,
}

impl Playlist {
    pub fn get_id(&self) -> &str {
        &self.rating_key
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn get_summary(&self) -> &str {
        &self.summary
    }

    pub fn get_length(&self) -> i32 {
        self.leaf_count as i32
    }

    pub fn get_duration(&self) -> u128 {
        self.duration.unwrap_or(0)
    }
}
