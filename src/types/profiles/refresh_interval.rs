use nutype::nutype;

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
        let refresh_interval = RefreshInterval::try_new(5).unwrap();

        assert_eq!(valid_refresh_interval, refresh_interval.into_inner());
    }

    #[test]
    fn test_invalid_refresh_interval() {
        let expected = Err(RefreshIntervalError::LessOrEqualViolated);
        let invalid_refresh_interval = 72_u32;
        let result = RefreshInterval::try_new(invalid_refresh_interval);
        assert_eq!(expected, result);
    }
}
