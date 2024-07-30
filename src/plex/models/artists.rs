use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::plex::plex_id::PlexId;
use crate::types::plex::plex_key::PlexKey;
use crate::types::Title;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    rating_key: PlexId,
    key: PlexKey,
    title: Title,
}

impl Artist {
    pub fn new(title: &str, id: &str, key: &str) -> Result<Self> {
        let rating_key = PlexId::try_new(id)?;
        let key = PlexKey::try_new(key)?;
        let title = Title::try_new(title)?;

        let artist = Self {
            rating_key,
            key,
            title,
        };

        Ok(artist)
    }

    pub fn get_id(&self) -> &str {
        &self.rating_key
    }

    pub fn get_key(&self) -> &str {
        &self.key
    }

    pub fn get_title(&self) -> &str {
        self.title.as_str()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    static ARTIST_TITLE: &str = "Rush";
    static ARTIST_ID: &str = "1041";
    static ARTIST_KEY: &str = "/library/metadata/1041/children";

    #[test]
    fn test_valid_artist() {
        let valid_artist = Artist {
            rating_key: PlexId::try_new(ARTIST_ID).unwrap(),
            key: PlexKey::try_new(ARTIST_KEY).unwrap(),
            title: Title::try_new(ARTIST_TITLE).unwrap(),
        };
        let test_artist = Artist::new(ARTIST_TITLE, ARTIST_ID, ARTIST_KEY).unwrap();

        assert_eq!(valid_artist.key, test_artist.key);
        assert_eq!(valid_artist.get_id(), test_artist.get_id());

        assert_eq!(valid_artist.rating_key, test_artist.rating_key);
        assert_eq!(valid_artist.get_key(), test_artist.get_key());

        assert_eq!(valid_artist.title, test_artist.title);
        assert_eq!(valid_artist.get_title(), test_artist.get_title());
    }

    #[test]
    #[should_panic]
    fn test_invalid_artist() {
        let _ = Artist::new("", "", "").unwrap();
    }
}
