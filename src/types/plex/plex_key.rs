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
    default = "/library/metadata/12345",
    validate(not_empty)
)]
pub struct PlexKey(String);

#[cfg(test)]
mod plex_key_tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_valid_plex_key() {
        let valid = "/library/metadata/129843";
        let plex_key = PlexKey::try_new(valid).unwrap();
        assert_eq!(valid, plex_key.as_ref())
    }

    #[test]
    fn test_invalid_plex_key_empty() {
        let expected = Err(PlexKeyError::NotEmptyViolated);
        let result = PlexKey::try_new("");
        assert_eq!(expected, result)
    }
}
