use std::fmt::{Display, Formatter};

use crate::plex::types::Guid;
use crate::types::Title;
use chrono::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    rating_key: String,
    pub key: String,
    pub parent_rating_key: String,
    pub grandparent_rating_key: String,
    pub guid: Guid,
    pub parent_guid: Guid,
    pub grandparent_guid: Guid,
    pub parent_studio: Option<String>,
    #[serde(alias = "type")]
    pub track_type: String,
    title: Title,
    pub parent_key: String,
    pub grandparent_key: String,
    grandparent_title: Title,
    parent_title: Title,
    pub summary: String,
    pub index: Option<u32>,
    pub parent_index: u32,
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
    original_title: Option<Title>,
    #[serde(alias = "Media")]
    pub media: Vec<Media>,
}

impl Track {
    pub fn id(&self) -> &str {
        &self.rating_key
    }

    pub fn title(&self) -> &str {
        self.title.as_ref()
    }

    pub fn album(&self) -> &str {
        self.parent_title.as_ref()
    }

    pub fn artist(&self) -> &str {
        match &self.original_title {
            Some(artist) => artist.as_ref(),
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
