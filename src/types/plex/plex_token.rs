use nutype::nutype;
use regex::Regex;
use std::sync::LazyLock;

static PLEX_TOKEN_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\w{9}-\w{10}$").unwrap());

#[nutype(
    derive(Clone, Debug, Default, Deserialize, Display, Serialize, AsRef, Deref, PartialEq),
    default = "PLEXPLEX1-TOKENTOKEN",
    validate(not_empty, regex = PLEX_TOKEN_REGEX)
)]
pub struct PlexToken(String);

#[cfg(test)]
mod plex_token_tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_valid_token() {
        let fake_plex_token = "NJlYINZmB-Hdy78xubjR";
        let plex_token = PlexToken::try_new(fake_plex_token).unwrap();
        assert_eq!(fake_plex_token, plex_token.into_inner())
    }

    #[test]
    fn test_invalid_token_empty() {
        let expected = Err(PlexTokenError::NotEmptyViolated);
        let result = PlexToken::try_new("");
        assert_eq!(expected, result)
    }

    #[test]
    fn text_invalid_token_regex() {
        let expected = Err(PlexTokenError::RegexViolated);

        let result = PlexToken::try_new("Three years later, the coffin was still full of Jello.");
        assert_eq!(expected, result);

        let result = PlexToken::try_new("COhwYWn9BjJpj8s54XbF");
        assert_eq!(expected, result);

        let result = PlexToken::try_new("^*!@GWObj-wZCeVg2lZ3");
        assert_eq!(expected, result);

        let result = PlexToken::try_new("s4MXW4pMzC-pxGIyBBdD");
        assert_eq!(expected, result);
    }
}
