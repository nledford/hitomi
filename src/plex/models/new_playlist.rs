use crate::plex::types::PlexId;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct NewPlaylist {
    #[serde(alias = "ratingKey")]
    pub rating_key: PlexId,
}
