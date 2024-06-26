use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename = "MediaContainer")]
pub struct SectionContainer {
    #[serde(alias = "Directory")]
    pub directory: Vec<Section>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct Section {
    pub title: String,
    #[serde(alias = "type")]
    pub plex_section_type: String,
    key: String,
}

impl Section {
    pub fn id(&self) -> &str {
        &self.key
    }

    pub fn is_type_music(&self) -> bool {
        self.plex_section_type == "artist"
    }
}
