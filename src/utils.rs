use chrono::Timelike;

use crate::profiles::types::RefreshInterval;

/// Constructs a `vec` of valid refresh minutes from a given refresh intervals
pub fn build_refresh_minutes(refresh_interval: &RefreshInterval) -> Vec<u32> {
    let interval: u32 = refresh_interval.clone().into_inner();

    (1..=60).filter(|i| i % interval == 0).collect()
}

/*pub async fn perform_refresh() -> bool {
    Utc::now().second() == 0 && state::get_any_profile_refresh().await
}*/

pub fn truncate_string(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;

    static EXPECTED_MINUTES: [u32; 12] = [5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55, 60];
    const VALID_INTERVAL: u32 = 5;
    const INVALID_INTERVAL: u32 = 10;

    #[test]
    fn test_build_refresh_minutes() {
        let interval = RefreshInterval::try_new(VALID_INTERVAL).unwrap();
        let minutes = build_refresh_minutes(&interval);
        assert_eq!(EXPECTED_MINUTES.to_vec(), minutes);
    }

    #[test]
    fn test_build_invalid_refresh_minutes() {
        let interval = RefreshInterval::try_new(INVALID_INTERVAL).unwrap();
        let minutes = build_refresh_minutes(&interval);
        assert_ne!(EXPECTED_MINUTES.to_vec(), minutes);
    }

    #[test]
    fn test_truncate_string() {
        let str = "It's not possible to convince a monkey to give you a banana by promising it infinite bananas when they die.";
        let truncated = truncate_string(str, 50);
        let expected = "It's not possible to convince a monkey to give you";

        assert_ne!(truncated, str);
        assert_eq!(expected, truncated);
    }
}
