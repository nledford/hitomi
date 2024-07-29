use serde::Deserialize;

use crate::types::plex::plex_id::PlexId;

#[derive(Debug, Default, Deserialize)]
pub struct NewPlaylist {
    #[serde(alias = "ratingKey")]
    pub rating_key: PlexId,
}
