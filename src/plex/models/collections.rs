use crate::plex::types::PlexId;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Collection {
    #[serde(alias = "ratingKey")]
    rating_key: PlexId,
    title: String,
}

impl Collection {
    pub fn get_id(&self) -> &str {
        &self.rating_key
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }
}
