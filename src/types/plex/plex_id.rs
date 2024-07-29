use std::sync::LazyLock;

use nutype::nutype;
use regex::Regex;

static PLEX_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9]{4,6}$").unwrap());

#[nutype(
    derive(Clone, Debug, Default, Deserialize, Display, Serialize, AsRef, Deref, PartialEq),
    default = "123456",
    validate(not_empty, len_char_max = 6, regex = PLEX_ID_REGEX)
)]
pub struct PlexId(String);

#[cfg(test)]
mod plex_id_tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_valid_plex_id() {
        let valid_id = "1234";
        let result = PlexId::try_new(valid_id).unwrap();
        assert_eq!(valid_id, result.into_inner());

        let valid_id = "12345";
        let result = PlexId::try_new(valid_id).unwrap();
        assert_eq!(valid_id, result.into_inner());

        let valid_id = "123456";
        let result = PlexId::try_new(valid_id).unwrap();
        assert_eq!(valid_id, result.into_inner());

        let valid_id = "666666";
        let result = PlexId::try_new(valid_id).unwrap();
        assert_eq!(valid_id, result.into_inner());

        let valid_id = "999999";
        let result = PlexId::try_new(valid_id).unwrap();
        assert_eq!(valid_id, result.into_inner());
    }

    #[test]
    fn test_invalid_plex_id_empty() {
        let expected = Err(PlexIdError::NotEmptyViolated);
        let result = PlexId::try_new("");
        assert_eq!(expected, result);
    }

    #[test]
    fn test_invalid_plex_id_length() {
        let expected = Err(PlexIdError::LenCharMaxViolated);
        let result = PlexId::try_new("1234567");
        assert_eq!(expected, result);

        let result =
            PlexId::try_new("It's important to remember to be aware of rampaging grizzly bears.");
        assert_eq!(expected, result);
    }

    #[test]
    fn test_invalid_plex_id_regex() {
        let expected = Err(PlexIdError::RegexViolated);

        let result = PlexId::try_new("0");
        assert_eq!(expected, result);

        let result = PlexId::try_new("123abc");
        assert_eq!(expected, result);

        let result = PlexId::try_new("abcdef");
        assert_eq!(expected, result);

        let result = PlexId::try_new("a@7)bc");
        assert_eq!(expected, result);
    }
}
