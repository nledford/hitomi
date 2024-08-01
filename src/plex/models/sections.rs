use crate::types::plex::plex_key::PlexKey;
use crate::types::Title;
use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename = "MediaContainer")]
pub struct SectionContainer {
    #[serde(alias = "Directory")]
    pub directory: Vec<Section>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct Section {
    title: Title,
    #[serde(alias = "type")]
    plex_section_type: String,
    key: PlexKey,
}

impl Section {
    pub fn id(&self) -> &str {
        &self.key
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn is_type_music(&self) -> bool {
        self.plex_section_type == "artist"
    }
}
