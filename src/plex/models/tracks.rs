use std::fmt::{Display, Formatter};

use chrono::{DateTime, Duration, TimeDelta, Timelike, Utc};
use serde::{Deserialize, Serialize};

use crate::plex::types::{Guid, PlexId, PlexKey};
use crate::types::Title;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    rating_key: PlexId,
    key: PlexKey,
    parent_rating_key: PlexId,
    grandparent_rating_key: PlexId,
    guid: Guid,
    parent_guid: Guid,
    grandparent_guid: Guid,
    // pub parent_studio: Option<String>,
    #[serde(alias = "type")]
    pub track_type: String,
    title: Title,
    pub parent_key: PlexKey,
    pub grandparent_key: PlexKey,
    grandparent_title: Title,
    parent_title: Title,
    // pub summary: String,
    pub index: Option<u32>,
    pub parent_index: u32,
    // rating_count: Option<i32>,
    user_rating: Option<f32>,
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
    // #[serde(alias = "Media")]
    // pub media: Vec<Media>,
}

impl Track {
    pub fn get_id(&self) -> &str {
        &self.rating_key
    }

    pub fn get_guid(&self) -> &str {
        self.guid.as_str()
    }

    pub fn get_track_title(&self) -> &str {
        self.title.as_ref()
    }

    pub fn get_track_album(&self) -> &str {
        self.parent_title.as_ref()
    }

    pub fn get_track_artist(&self) -> &str {
        match &self.original_title {
            Some(artist) => artist.as_ref(),
            None => &self.grandparent_title,
        }
            .trim()
    }

    pub fn get_artist_id(&self) -> &str {
        self.grandparent_rating_key.as_str()
    }

    pub fn get_artist_guid(&self) -> &str {
        self.grandparent_guid.as_str()
    }

    pub fn get_track_duration(&self) -> i64 {
        self.duration.unwrap_or(0)
    }

    pub fn get_track_duration_timedelta(&self) -> TimeDelta {
        TimeDelta::milliseconds(self.get_track_duration())
    }

    pub fn get_last_played(&self) -> i64 {
        self.last_viewed_at.unwrap_or(0)
    }

    pub fn get_last_played_str(&self) -> String {
        DateTime::from_timestamp(self.get_last_played(), 0)
            .unwrap()
            .naive_local()
            .format("%F")
            .to_string()
    }

    pub fn get_last_played_year_and_month(&self) -> String {
        DateTime::from_timestamp(self.get_last_played(), 0)
            .unwrap()
            .naive_local()
            .format("%Y-%m")
            .to_string()
    }

    pub fn get_played_today(&self) -> bool {
        self.get_last_played() >= Utc::now()
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .timestamp()
    }

    pub fn get_played_within_last_day(&self) -> bool {
        let one_day_ago = (Utc::now() - Duration::days(1)).timestamp();
        self.get_last_played() >= one_day_ago
    }

    pub fn get_plays(&self) -> i32 {
        self.view_count.unwrap_or(0)
    }

    pub fn get_has_never_been_played(&self) -> bool {
        self.get_plays() == 0 || self.get_last_played() == 0
    }

    pub fn get_rating(&self) -> i32 {
        let rating = self.user_rating.unwrap_or_default();

        (rating / 2_f32).floor() as i32
    }
}

impl Display for Track {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut str = String::default();

        str += &format!("{} ", self.title);
        str += &format!("{} ", self.get_track_artist());
        str += &format!("{} ", self.get_track_album());
        str += &format!("{} ", self.get_plays());
        str += &format!("{} ", self.get_last_played_str());

        write!(f, "{str}")
    }
}

/*
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    id: i64,
    bitrate: Option<i64>,
    duration: Option<i64>,
    audio_channels: i64,
    audio_codec: String,
    #[serde(alias = "Part")]
    part: Vec<Part>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    id: i64,
    key: String,
    duration: Option<i64>,
    file: String,
    size: i64,
}
*/
