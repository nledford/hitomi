use crate::plex::types::PlexId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SubType {
    #[default]
    Artist,
    Track,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Collection {
    #[serde(alias = "ratingKey")]
    rating_key: PlexId,
    title: String,
    subtype: SubType,
}

impl Collection {
    pub fn get_id(&self) -> &str {
        &self.rating_key
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn get_subtype(&self) -> &SubType {
        &self.subtype
    }
}
