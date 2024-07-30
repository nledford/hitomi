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
