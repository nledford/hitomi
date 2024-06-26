use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Artist {
    rating_key: String,
    pub key: String,
    pub guid: String,
    pub title: String,
    #[serde(alias = "titleSort")]
    pub title_sort: String,
}

impl Artist {
    pub fn id(&self) -> &str {
        &self.rating_key
    }
}
