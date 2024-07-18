use std::fmt::{Display, Formatter};

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::profiles::SectionType;

#[allow(dead_code)]
#[derive(Builder, Clone, Debug, Default, Deserialize, PartialEq, Serialize, sqlx::FromRow)]
pub struct ProfileSection {
    /// The primary key in the database
    #[builder(setter(skip))]
    profile_section_id: i32,
    /// The foreign key linking to the profile in the database
    #[builder(setter(skip))]
    profile_id: i32,
    /// Deduplicate tracks by its `guid`, so that the exact same track that appears on
    /// multiple albums (e.g., a studio album and a Greatest Hits album) only appears once in
    /// the resulting playlist.
    deduplicate_tracks_by_guid: bool,
    deduplicate_tracks_by_title_and_artist: bool,
    enabled: bool,
    /// Caps the number of tracks by an artist that can appear in a single playlist.
    /// A value of `0` allows for an unlimited number of tracks.
    maximum_tracks_by_artist: u32,
    minimum_track_rating: u32,
    randomize_tracks: bool,
    section_type: SectionType,
    sorting: String,
}

impl ProfileSection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_profile_section_id(&self) -> i32 {
        self.profile_section_id
    }

    pub fn get_profile_id(&self) -> i32 {
        self.profile_id
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_section_type(&self) -> SectionType {
        self.section_type
    }

    pub fn is_section_type(&self, section_type: SectionType) -> bool {
        self.get_section_type() == section_type
    }

    pub fn is_unplayed_section(&self) -> bool {
        self.is_section_type(SectionType::Unplayed)
    }

    pub fn is_least_played_section(&self) -> bool {
        self.is_section_type(SectionType::LeastPlayed)
    }

    pub fn is_oldest_section(&self) -> bool {
        self.is_section_type(SectionType::Oldest)
    }

    pub fn get_minimum_track_rating(&self) -> u32 {
        if self.minimum_track_rating <= 1 {
            return 0;
        }
        self.minimum_track_rating
    }

    pub fn get_minimum_track_rating_adjusted(&self) -> u32 {
        if self.get_minimum_track_rating() <= 1 {
            return 0;
        }
        (self.get_minimum_track_rating() - 1) * 2
    }

    pub fn get_sorting_vec(&self) -> Vec<&str> {
        self.sorting.split(',').collect::<_>()
    }

    pub fn get_sorting(&self) -> &str {
        &self.sorting
    }

    pub fn get_deduplicate_tracks_by_guid(&self) -> bool {
        self.deduplicate_tracks_by_guid
    }

    pub fn get_deduplicate_tracks_by_title_and_artist(&self) -> bool {
        self.deduplicate_tracks_by_title_and_artist
    }

    pub fn get_maximum_tracks_by_artist(&self) -> u32 {
        self.maximum_tracks_by_artist
    }

    pub fn get_randomize_tracks(&self) -> bool {
        self.randomize_tracks
    }
}

impl Display for ProfileSection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut str = format!("  {}", self.section_type);
        str += &format!(
            "\n    Enabled:                                {}",
            self.enabled
        );
        str += &format!(
            "\n    Deduplicate tracks by GUID:             {}",
            self.deduplicate_tracks_by_guid
        );
        str += &format!(
            "\n    Deduplicate tracks by title and artist: {}",
            self.deduplicate_tracks_by_title_and_artist
        );
        str += &format!(
            "\n    Maximum tracks by artist:               {}",
            if self.maximum_tracks_by_artist == 0 {
                "Unlimited".to_string()
            } else {
                format!("{} track(s)", self.maximum_tracks_by_artist)
            }
        );
        str += &format!(
            "\n    Minimum track rating:                   {} stars",
            self.minimum_track_rating
        );
        str += &format!(
            "\n    Sorting:                                {}",
            self.sorting
        );

        writeln!(f, "{str}")
    }
}
