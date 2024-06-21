use nutype::nutype;
use once_cell::sync::Lazy;
use regex::Regex;

static PROFILE_SOURCE_ID_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d+$").unwrap());

#[nutype(
    derive(
        Clone,
        Default,
        Debug,
        Deserialize,
        Display,
        PartialEq,
        Serialize,
        AsRef,
        Deref
    ),
    default = "New Profile",
    validate(not_empty, len_char_max = 25)
)]
pub struct ProfileTitle(String);

#[cfg(test)]
mod profile_title_tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_valid_title() {
        let valid_title = "Valid Title";
        let title = ProfileTitle::new(valid_title).unwrap();
        assert_eq!(valid_title, title.into_inner())
    }

    #[test]
    fn test_invalid_title_blank() {
        let expected = Err(ProfileTitleError::NotEmptyViolated);
        let result = ProfileTitle::new("");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_invalid_title_too_long() {
        let expected = Err(ProfileTitleError::LenCharMaxViolated);
        let invalid_title = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";
        let result = ProfileTitle::new(invalid_title);
        assert_eq!(expected, result)
    }
}

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
