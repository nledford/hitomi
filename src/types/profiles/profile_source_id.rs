use std::sync::LazyLock;

use nutype::nutype;
use regex::Regex;

static PROFILE_SOURCE_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+$").unwrap());

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
        let profile_source_id = ProfileSourceId::try_new(valid_id).unwrap();
        assert_eq!(valid_id, profile_source_id.into_inner());
    }

    #[test]
    fn test_invalid_source_id_blank() {
        let expected = Err(ProfileSourceIdError::NotEmptyViolated);

        let invalid_id = "";
        let result = ProfileSourceId::try_new(invalid_id);
        assert_eq!(expected, result);
    }

    #[test]
    fn test_invalid_profile_source_id_alpha() {
        let expected = Err(ProfileSourceIdError::RegexViolated);

        let invalid_id = "123abc";
        let result = ProfileSourceId::try_new(invalid_id);
        assert_eq!(expected, result);

        let invalid_id = "abc123";
        let result = ProfileSourceId::try_new(invalid_id);
        assert_eq!(expected, result);

        let invalid_id = "abcdefg";
        let result = ProfileSourceId::try_new(invalid_id);
        assert_eq!(expected, result);
    }
}
