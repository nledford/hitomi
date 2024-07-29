use nutype::nutype;
use regex::Regex;
use std::sync::LazyLock;

use crate::profiles::SectionType;

static PROFILE_SECTION_SORT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(([A-Za-z]+:?[A-Za-z]*),?)+$").unwrap());

#[nutype(
    derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, AsRef, Deref),
    default = "viewCount,lastViewedAt",
    validate(regex = PROFILE_SECTION_SORT_REGEX)
)]
pub struct ProfileSectionSort(String);

impl ProfileSectionSort {
    pub fn default_from(section_type: SectionType) -> Self {
        let sort = match section_type {
            SectionType::Unplayed => vec![
                "userRating:desc",
                "viewCount",
                "lastViewedAt",
                "guid",
                "mediaBitrate:desc",
            ],
            SectionType::LeastPlayed => {
                vec!["viewCount", "lastViewedAt", "guid", "mediaBitrate:desc"]
            }
            SectionType::Oldest => vec!["lastViewedAt", "viewCount", "guid", "mediaBitrate:desc"],
        }
        .join(",");

        Self::try_new(sort).unwrap()
    }
}
