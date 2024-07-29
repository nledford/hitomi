use nutype::nutype;

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

        assert_eq!(
            valid_track,
            Guid::try_new(valid_track).unwrap().into_inner()
        );
        assert_eq!(
            valid_album,
            Guid::try_new(valid_album).unwrap().into_inner()
        );
        assert_eq!(
            valid_artist,
            Guid::try_new(valid_artist).unwrap().into_inner()
        );
    }

    #[test]
    fn test_invalid_guid_empty() {
        let expected = Err(GuidError::NotEmptyViolated);
        let invalid = "";
        let result = Guid::try_new(invalid);
        assert_eq!(expected, result)
    }
}
