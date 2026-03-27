use std::time::{Duration, Instant};

pub fn frame_interval_from_env(var_name: &str, default: u64) -> u64 {
    std::env::var(var_name)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|&v| v > 0)
        .unwrap_or(default)
}

pub fn should_log_every_n_frames(frame: u64, interval: u64) -> bool {
    interval > 0 && frame.is_multiple_of(interval)
}

pub fn should_log_after(last: &mut Instant, interval: Duration) -> bool {
    if last.elapsed() >= interval {
        *last = Instant::now();
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_log_every_n_frames() {
        assert!(!should_log_every_n_frames(1, 60));
        assert!(should_log_every_n_frames(60, 60));
    }

    #[test]
    fn test_frame_interval_from_env_default() {
        let value = frame_interval_from_env("REDRING_UNSET_TEST_VAR", 120);
        assert_eq!(value, 120);
    }
}
