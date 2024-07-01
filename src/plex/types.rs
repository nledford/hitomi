use nutype::nutype;
use once_cell::sync::Lazy;
use regex::Regex;

static PLAYLIST_ID_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[0-9]{6}$").unwrap());
static PLEX_TOKEN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[\w\d]{9}-[\w\d]{10}$").unwrap());

// SOURCE: https://stackoverflow.com/a/3809435
static PLEX_URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https?://(www\.)?[-a-zA-Z0-9@:%._+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_+.~#?&/=]*)").unwrap()
});

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
    default = "plex://track/0a0a0a0a0a0a0a0a0a0a0a0a",
    sanitize(trim),
    validate(not_empty)
)]
pub struct Guid(String);

#[cfg(test)]
mod guid_tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_valid_guid() {
        let valid_track = "plex://track/608bcb5f0f0b9c002cf4cd16";
        let valid_album = "plex://album/608bbd7b295725002cd9c7cc";
        let valid_artist = "plex://artist/5fb686acfb665dfcb10d25c9";

        assert_eq!(valid_track, Guid::new(valid_track).unwrap().into_inner());
        assert_eq!(valid_album, Guid::new(valid_album).unwrap().into_inner());
        assert_eq!(valid_artist, Guid::new(valid_artist).unwrap().into_inner());
    }

    #[test]
    fn test_invalid_guid_empty() {
        let expected = Err(GuidError::NotEmptyViolated);
        let invalid = "";
        let result = Guid::new(invalid);
        assert_eq!(expected, result)
    }
}

#[nutype(
    derive(Clone, Debug, Default, Deserialize, Display, Serialize, AsRef, Deref, PartialEq),
    default = "123456",
    validate(not_empty, len_char_max = 6, regex = PLAYLIST_ID_REGEX)
)]
pub struct PlaylistId(String);

#[cfg(test)]
mod playlist_id_tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_valid_playlist_id() {
        let valid_id = "123456";
        let result = PlaylistId::new(valid_id).unwrap();
        assert_eq!(valid_id, result.into_inner());

        let valid_id = "666666";
        let result = PlaylistId::new(valid_id).unwrap();
        assert_eq!(valid_id, result.into_inner());

        let valid_id = "999999";
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
    fn test_invalid_playlist_id_length() {
        let expected = Err(PlaylistIdError::LenCharMaxViolated);
        let result = PlaylistId::new("1234567");
        assert_eq!(expected, result);

        let result =
            PlaylistId::new("It's important to remember to be aware of rampaging grizzly bears.");
        assert_eq!(expected, result);
    }

    #[test]
    fn test_invalid_playlist_id_regex() {
        let expected = Err(PlaylistIdError::RegexViolated);

        let result = PlaylistId::new("0");
        assert_eq!(expected, result);

        let result = PlaylistId::new("123abc");
        assert_eq!(expected, result);

        let result = PlaylistId::new("abcdef");
        assert_eq!(expected, result);

        let result = PlaylistId::new("a@7)bc");
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
