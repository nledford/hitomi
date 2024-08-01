use std::fmt::{Display, Formatter};

use jiff::tz::TimeZone;
use jiff::{Timestamp, ToSpan, Zoned};
use serde::{Deserialize, Serialize};

use crate::types::plex::guid::Guid;
use crate::types::plex::plex_id::PlexId;
use crate::types::plex::plex_key::PlexKey;
use crate::types::Title;
use crate::utils;

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
    #[serde(alias = "type")]
    track_type: String,
    title: Title,
    parent_key: PlexKey,
    grandparent_key: PlexKey,
    grandparent_title: Title,
    parent_title: Title,
    index: Option<u32>,
    parent_index: u32,
    user_rating: Option<f32>,
    view_count: Option<i32>,
    last_viewed_at: Option<i64>,
    parent_year: Option<i32>,
    /// Duration is in milliseconds
    duration: Option<i64>,
    original_title: Option<Title>,
    #[serde(alias = "Media")]
    pub media: Vec<Media>,
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

    /// Duration is in milliseconds
    pub fn get_track_duration(&self) -> i64 {
        self.duration.unwrap_or(0)
    }

    /// In milliseconds
    pub fn get_last_played(&self) -> Timestamp {
        if let Some(last_viewed_at) = self.last_viewed_at {
            Timestamp::from_millisecond(last_viewed_at).unwrap_or_default()
        } else {
            Timestamp::default()
        }
    }

    fn get_last_played_datetime(&self) -> Zoned {
        self.get_last_played().to_zoned(TimeZone::system())
    }

    pub fn get_last_played_str(&self) -> String {
        self.get_last_played().strftime("%F").to_string()
    }

    pub fn get_last_played_year_and_month(&self) -> String {
        self.get_last_played().strftime("%Y-%m").to_string()
    }

    pub fn get_played_within_last_day(&self) -> bool {
        let last_played = self.get_last_played_datetime();
        let now = utils::get_current_datetime();
        let thirty_six_hours_ago = now.checked_sub(36.hours());

        if let Ok(thirty_six_hours_ago) = thirty_six_hours_ago {
            last_played >= thirty_six_hours_ago
        } else {
            false
        }
    }

    pub fn get_plays(&self) -> i32 {
        self.view_count.unwrap_or(0)
    }

    pub fn get_has_never_been_played(&self) -> bool {
        self.get_plays() == 0 || self.get_last_played() == Timestamp::default()
    }

    pub fn get_rating(&self) -> i32 {
        let rating = self.user_rating.unwrap_or_default();

        (rating / 2.0).floor() as i32
    }

    pub fn get_bitrate(&self) -> i64 {
        match self.media.first() {
            Some(media) => media.bitrate.unwrap_or(0),
            None => 0,
        }
    }

    pub fn get_title_and_artist_sort_key(&self) -> (String, String) {
        (
            self.get_track_title().to_string(),
            self.get_track_artist().to_string(),
        )
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    id: i64,
    bitrate: Option<i64>,
    duration: Option<i64>,
    audio_channels: i64,
    audio_codec: String,
}
