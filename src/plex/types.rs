use nutype::nutype;
use once_cell::sync::Lazy;
use regex::Regex;

static PLAYLIST_ID_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[1-9]+\d*$").unwrap());
static PLEX_TOKEN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[\w\d]{9}-[\w\d]{10}$").unwrap());

// SOURCE: https://stackoverflow.com/a/3809435
static PLEX_URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https?://(www\.)?[-a-zA-Z0-9@:%._+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_+.~#?&/=]*)").unwrap()
});

#[nutype(
    derive(Clone, Debug, Default, Deserialize, Display, Serialize, AsRef, Deref, PartialEq),
    default = "1",
    validate(not_empty, regex = PLAYLIST_ID_REGEX)
)]
pub struct PlaylistId(String);

#[cfg(test)]
mod playlist_id_tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_valid_playlist_id() {
        let valid_id = "1";
        let result = PlaylistId::new(valid_id).unwrap();
        assert_eq!(valid_id, result.into_inner());

        let valid_id = "22";
        let result = PlaylistId::new(valid_id).unwrap();
        assert_eq!(valid_id, result.into_inner());

        let valid_id = "333";
        let result = PlaylistId::new(valid_id).unwrap();
        assert_eq!(valid_id, result.into_inner());

        let valid_id = "4444";
        let result = PlaylistId::new(valid_id).unwrap();
        assert_eq!(valid_id, result.into_inner());

        let valid_id = "55555";
        let result = PlaylistId::new(valid_id).unwrap();
        assert_eq!(valid_id, result.into_inner());
    }

    #[test]
    fn test_invalid_playlist_id_empty() {
        let expected = Err(PlaylistIdError::NotEmptyViolated);
        let result = PlaylistId::new("");
        assert_eq!(expected, result);
    }

    #[test]
    fn test_invalid_playlist_id_regex() {
        let expected = Err(PlaylistIdError::RegexViolated);

        let result = PlaylistId::new("0");
        assert_eq!(expected, result);

        let result =
            PlaylistId::new("It's important to remember to be aware of rampaging grizzly bears.");
        assert_eq!(expected, result);

        let result = PlaylistId::new("123abc");
        assert_eq!(expected, result);

        let result = PlaylistId::new("abcdefg");
        assert_eq!(expected, result);
    }
}

#[nutype(
    derive(
        Clone,
        Debug,
        Default,
        Deserialize,
        Display,
        Serialize,
        AsRef,
        Deref,
        PartialEq
    ),
    default = "New Playlist",
    validate(not_empty)
)]
pub struct PlaylistTitle(String);

#[cfg(test)]
mod playlist_title_tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_playlist_title_valid() {
        let valid = "Valid Playlist Title";
        let result = PlaylistTitle::new(valid).unwrap();
        assert_eq!(valid, result.into_inner());
    }

    #[test]
    fn test_invalid_playlist_title_empty() {
        let expected = Err(PlaylistTitleError::NotEmptyViolated);
        let invalid = "";
        let result = PlaylistTitle::new(invalid);
        assert_eq!(expected, result);
    }
}

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
        let plex_token = PlexToken::new(fake_plex_token).unwrap();
        assert_eq!(fake_plex_token, plex_token.into_inner())
    }

    #[test]
    fn test_invalid_token_empty() {
        let expected = Err(PlexTokenError::NotEmptyViolated);
        let result = PlexToken::new("");
        assert_eq!(expected, result)
    }

    #[test]
    fn text_invalid_token_regex() {
        let expected = Err(PlexTokenError::RegexViolated);

        let result = PlexToken::new("Three years later, the coffin was still full of Jello.");
        assert_eq!(expected, result);

        let result = PlexToken::new("COhwYWn9BjJpj8s54XbF");
        assert_eq!(expected, result);

        let result = PlexToken::new("^*!@GWObj-wZCeVg2lZ3");
        assert_eq!(expected, result);

        let result = PlexToken::new("s4MXW4pMzC-pxGIyBBdD");
        assert_eq!(expected, result);
    }
}

#[nutype(
    derive(Clone, Debug, Default, Deserialize, Display, Serialize, AsRef, Deref, PartialEq),
    default = "http://127.0.0.1:32400",
    validate(not_empty, regex = PLEX_URL_REGEX)
)]
pub struct PlexUrl(String);

#[cfg(test)]
mod plex_url_tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_valid_plex_url() {
        let valid = "http://127.0.0.1:32400";
        let result = PlexUrl::new(valid).unwrap();
        assert_eq!(valid, result.into_inner());

        let valid = "http://127.0.0.1:2112";
        let result = PlexUrl::new(valid).unwrap();
        assert_eq!(valid, result.into_inner());

        let valid = "https://plex.domain.com";
        let result = PlexUrl::new(valid).unwrap();
        assert_eq!(valid, result.into_inner());

        let valid = "https://domain.com/plex";
        let result = PlexUrl::new(valid).unwrap();
        assert_eq!(valid, result.into_inner());
    }

    #[test]
    fn test_invalid_plex_url_empty() {
        let expected = Err(PlexUrlError::NotEmptyViolated);
        let result = PlexUrl::new("");
        assert_eq!(expected, result);
    }

    #[test]
    fn text_invalid_plex_url_regex() {
        let expected = Err(PlexUrlError::RegexViolated);

        let result = PlexUrl::new("He swore he just saw his sushi move.");
        assert_eq!(result, expected);

        let result = PlexUrl::new("me@thegoogle.com");
        assert_eq!(result, expected);

        let result = PlexUrl::new("htt://127.0.0.1:32400");
        assert_eq!(result, expected);

        let result = PlexUrl::new("127.0.0.1:32400");
        assert_eq!(result, expected);
    }
}
