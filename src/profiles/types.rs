use nutype::nutype;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::profiles::SectionType;

static PROFILE_SOURCE_ID_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d+$").unwrap());
static PROFILE_SECTION_SORT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(([A-Za-z]+:?[A-Za-z]*),?)+$").unwrap());

#[nutype(
    derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize, AsRef, Deref),
    default = "0",
    validate(not_empty, regex = PROFILE_SOURCE_ID_REGEX)
)]
pub struct ProfileSourceId(String);

#[cfg(test)]
mod profile_source_id_tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_valid_profile_source_id() {
        let valid_id = "5";
        let profile_source_id = ProfileSourceId::new(valid_id).unwrap();
        assert_eq!(valid_id, profile_source_id.into_inner());
    }

    #[test]
    fn test_invalid_source_id_blank() {
        let expected = Err(ProfileSourceIdError::NotEmptyViolated);

        let invalid_id = "";
        let result = ProfileSourceId::new(invalid_id);
        assert_eq!(expected, result);
    }

    #[test]
    fn test_invalid_profile_source_id_alpha() {
        let expected = Err(ProfileSourceIdError::RegexViolated);

        let invalid_id = "123abc";
        let result = ProfileSourceId::new(invalid_id);
        assert_eq!(expected, result);

        let invalid_id = "abc123";
        let result = ProfileSourceId::new(invalid_id);
        assert_eq!(expected, result);

        let invalid_id = "abcdefg";
        let result = ProfileSourceId::new(invalid_id);
        assert_eq!(expected, result);
    }
}

#[nutype(
    default = 2,
    validate(less_or_equal = 60),
    derive(
        Clone,
        Debug,
        Default,
        Deserialize,
        Display,
        PartialEq,
        Serialize,
        AsRef,
        Deref
    )
)]
pub struct RefreshInterval(u32);

#[cfg(test)]
mod refresh_interval_tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_valid_refresh_interval() {
        let valid_refresh_interval = 5_u32;
        let refresh_interval = RefreshInterval::new(5).unwrap();

        assert_eq!(valid_refresh_interval, refresh_interval.into_inner());
    }

    #[test]
    fn test_invalid_refresh_interval() {
        let expected = Err(RefreshIntervalError::LessOrEqualViolated);
        let invalid_refresh_interval = 72_u32;
        let result = RefreshInterval::new(invalid_refresh_interval);
        assert_eq!(expected, result);
    }
}

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
