use nutype::nutype;
use serde::{Deserialize, Serialize};

use crate::types::plex::plex_id::PlexId;
use crate::types::plex::plex_key::PlexKey;
use crate::types::Title;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    rating_key: PlexId,
    key: PlexKey,
    // guid: String,
    title: Title,
    // #[serde(alias = "titleSort")]
    // title_sort: Option<Title>,
}

impl Artist {
    pub fn get_id(&self) -> &str {
        &self.rating_key
    }

    pub fn get_key(&self) -> &str {
        &self.key
    }

    // pub fn get_guid(&self) -> &str {
    //     &self.guid
    // }

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
        let artist_title = ArtistTitle::try_new(valid).unwrap();
        assert_eq!(valid, artist_title.as_str());
    }

    #[test]
    fn test_invalid_artist_title_empty() {
        let expected = Err(ArtistTitleError::NotEmptyViolated);
        let invalid = "";
        let result = ArtistTitle::try_new(invalid);
        assert_eq!(expected, result)
    }
}
