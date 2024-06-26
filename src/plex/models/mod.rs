use serde::Deserialize;

use crate::plex::models::sections::SectionContainer;

pub mod artists;
pub mod collections;
pub mod new_playlist;
pub mod playlists;
pub mod sections;
pub mod tracks;

pub type PlexResponse<T> = MediaContainerWrapper<MediaContainer<T>>;
pub type SectionResponse = MediaContainerWrapper<SectionContainer>;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaContainerWrapper<T> {
    #[serde(rename = "MediaContainer")]
    pub media_container: T,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaContainer<T> {
    pub size: Option<i32>,
    #[serde(alias = "Metadata")]
    pub metadata: T,
}
