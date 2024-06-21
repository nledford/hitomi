use crate::profiles::types::RefreshInterval;

/// Constructs a `vec` of valid refresh minutes from a given refresh intervals
pub fn build_refresh_minutes(refresh_interval: &RefreshInterval) -> Vec<u32> {
    let interval: u32 = refresh_interval.clone().into_inner();

    (1..=60).filter(|i| i % interval == 0).collect()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;

    static EXPECTED_MINUTES: [u32; 12] = [5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55, 60];

    #[test]
    fn test_build_refresh_minutes() {
        let interval = RefreshInterval::new(5).unwrap();
        let minutes = build_refresh_minutes(&interval);
        assert_eq!(EXPECTED_MINUTES.to_vec(), minutes);
    }

    #[test]
    fn test_build_invalid_refresh_minutes() {
        let interval = RefreshInterval::new(10).unwrap();
        let minutes = build_refresh_minutes(&interval);
        assert_ne!(EXPECTED_MINUTES.to_vec(), minutes);
    }
}
