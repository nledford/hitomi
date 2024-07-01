use nutype::nutype;

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
    default = "New Profile/Playlist",
    validate(not_empty, len_char_max = 50)
)]
pub struct Title(String);

#[cfg(test)]
mod profile_title_tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_valid_title() {
        let valid_title = "Valid Title";
        let title = Title::new(valid_title).unwrap();
        assert_eq!(valid_title, title.into_inner())
    }

    #[test]
    fn test_invalid_title_blank() {
        let expected = Err(TitleError::NotEmptyViolated);
        let result = Title::new("");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_invalid_title_too_long() {
        let expected = Err(TitleError::LenCharMaxViolated);
        let invalid_title = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";
        let result = Title::new(invalid_title);
        assert_eq!(expected, result)
    }
}
