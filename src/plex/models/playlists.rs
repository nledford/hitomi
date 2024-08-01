use serde::Deserialize;

use crate::types::plex::plex_id::PlexId;
use crate::types::plex::plex_key::PlexKey;
use crate::types::Title;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    rating_key: PlexId,
    key: PlexKey,
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

    pub fn get_key(&self) -> &str {
        &self.key
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn get_summary(&self) -> &str {
        &self.summary
    }

    pub fn get_item_count(&self) -> i32 {
        self.leaf_count as i32
    }

    pub fn is_empty(&self) -> bool {
        self.get_item_count() == 0
    }

    pub fn get_duration(&self) -> u128 {
        self.duration.unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_is_empty() {
        let playlist = Playlist::default();
        assert_eq!(playlist.is_empty(), true);

        let playlist = Playlist {
            leaf_count: 10,
            ..Default::default()
        };
        assert_eq!(playlist.is_empty(), false);
    }
}
