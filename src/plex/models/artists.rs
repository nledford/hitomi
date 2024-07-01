use nutype::nutype;
use serde::{Deserialize, Serialize};

use crate::plex::types::{PlexId, PlexKey};
use crate::types::Title;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Artist {
    rating_key: PlexId,
    key: PlexKey,
    pub guid: String,
    pub title: Title,
    #[serde(alias = "titleSort")]
    pub title_sort: String,
}

impl Artist {
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

#[nutype(
    derive(Clone, Debug, Serialize, Deserialize, PartialEq, AsRef, Deref),
    validate(not_empty)
)]
pub struct ArtistTitle(String);

#[cfg(test)]
mod test_artist_title {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_valid_artist_title() {
        let valid = "Rush";
        let artist_title = ArtistTitle::new(valid).unwrap();
        assert_eq!(valid, artist_title.as_str());
    }

    #[test]
    fn test_invalid_artist_title_empty() {
        let expected = Err(ArtistTitleError::NotEmptyViolated);
        let invalid = "";
        let result = ArtistTitle::new(invalid);
        assert_eq!(expected, result)
    }
}
