use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename = "MediaContainer")]
pub struct SectionContainer {
    #[serde(alias = "Directory")]
    pub directory: Vec<Section>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct Section {
    title: String,
    #[serde(alias = "type")]
    plex_section_type: String,
    key: String,
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
