use std::fmt::{Display, Formatter};

use chrono::DateTime;
use serde::{Deserialize, Serialize};

use crate::plex::types::PlaylistTitle;

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

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub rating_key: String,
    pub key: String,
    pub guid: String,
    #[serde(alias = "type")]
    pub item_type: String,
    pub title: PlaylistTitle,
    pub summary: String,
    pub smart: bool,
    pub playlist_type: String,
    // pub composite: String,
    pub icon: Option<String>,
    pub view_count: i32,
    pub last_viewed_at: u128,
    pub duration: Option<u128>,
    pub leaf_count: i32,
    pub added_at: u128,
    pub updated_at: u128,
}

impl Playlist {
    pub fn get_title(&self) -> &str {
        &self.title
    }
}

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    rating_key: String,
    pub key: String,
    pub parent_rating_key: String,
    pub grandparent_rating_key: String,
    pub guid: String,
    pub parent_guid: String,
    pub grandparent_guid: String,
    pub parent_studio: Option<String>,
    #[serde(alias = "type")]
    pub track_type: String,
    title: String,
    pub parent_key: String,
    pub grandparent_key: String,
    grandparent_title: String,
    parent_title: String,
    pub summary: String,
    pub index: Option<i32>,
    pub parent_index: i32,
    // rating_count: Option<i32>,
    user_rating: f32,
    view_count: Option<i32>,
    last_viewed_at: Option<i64>,
    // pub last_rated_at: Option<i64>,
    parent_year: Option<i32>,
    // pub thumb: Option<String>,
    // pub art: Option<String>,
    // pub parent_thumb: Option<String>,
    // pub grandparent_thumb: Option<String>,
    // pub grandparent_art: Option<String>,
    duration: Option<i64>,
    // added_at: Option<i64>,
    // updated_at: Option<i64>,
    skip_count: Option<i32>,
    // pub music_analysis_version: Option<String>,
    original_title: Option<String>,
    #[serde(alias = "Media")]
    pub media: Vec<Media>,
}

impl Track {
    pub fn id(&self) -> &str {
        &self.rating_key
    }

    pub fn title(&self) -> &str {
        self.title.trim()
    }

    pub fn album(&self) -> &str {
        self.parent_title.trim()
    }

    pub fn artist(&self) -> &str {
        match &self.original_title {
            Some(artist) => artist,
            None => &self.grandparent_title,
        }
        .trim()
    }

    pub fn artist_guid(&self) -> &str {
        &self.grandparent_guid
    }

    pub fn duration(&self) -> i64 {
        self.duration.unwrap_or(0)
    }

    pub fn last_played(&self) -> i64 {
        self.last_viewed_at.unwrap_or(0)
    }

    pub fn last_played_fmt(&self) -> String {
        DateTime::from_timestamp(self.last_played(), 0)
            .unwrap()
            .naive_local()
            .format("%Y-%m-%d")
            .to_string()
    }

    pub fn plays(&self) -> i32 {
        self.view_count.unwrap_or(0)
    }

    pub fn never_played(&self) -> bool {
        self.plays() == 0 || self.last_played() == 0
    }

    pub fn rating(&self) -> i32 {
        (self.user_rating / 2_f32).floor() as i32
    }
}

impl Display for Track {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut str = String::default();

        str += &format!("{} ", self.title);
        str += &format!("{} ", self.artist());
        str += &format!("{} ", self.album());
        str += &format!("{} ", self.plays());
        str += &format!("{} ", self.last_played_fmt());

        write!(f, "{str}")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub id: i64,
    pub bitrate: Option<i64>,
    pub duration: Option<i64>,
    pub audio_channels: i64,
    pub audio_codec: String,
    #[serde(alias = "Part")]
    pub part: Vec<Part>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    pub id: i64,
    pub key: String,
    pub duration: Option<i64>,
    pub file: String,
    pub size: i64,
}

impl Playlist {
    pub fn id(&self) -> &str {
        &self.rating_key
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Collection {
    #[serde(alias = "ratingKey")]
    pub rating_key: String,
    pub title: String,
}

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

#[derive(Debug, Default, Deserialize)]
pub struct NewPlaylist {
    #[serde(alias = "ratingKey")]
    pub rating_key: String,
}
