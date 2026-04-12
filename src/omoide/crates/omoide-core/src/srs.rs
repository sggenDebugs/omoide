use omoide_env::*;
use omoide_format::schema::SrsState;
use std::time::{SystemTime, UNIX_EPOCH};

const SUCCESS_MULTIPLIER: f32 = 1.5;
const FAILURE_DIVISOR: f32 = 2.0;
const MAX_INTERVAL_HOURS: f32 = SRS_DEFAULT_INTERVAL_HOURS * 365.0;

/// Calculates the next required interval based on success or failure.
pub fn next_interval(current_interval: f32, success: bool) -> f32 {
    if success {
        let next = current_interval * SUCCESS_MULTIPLIER;
        // Cap the maximum interval (e.g., to ~1 year for safety against never rehearsing again)
        if next.is_nan() || next > MAX_INTERVAL_HOURS {
            MAX_INTERVAL_HOURS
        } else {
            next
        }
    } else {
        // Punish failure by halving the interval, but never drop below initial 12.0 hours.
        let next = current_interval / FAILURE_DIVISOR;
        if next.is_nan() || next < SRS_DEFAULT_INTERVAL_HOURS {
            SRS_DEFAULT_INTERVAL_HOURS
        } else {
            next
        }
    }
}

/// Checks if an Emergency Access Rehearsal is mathematically due given the current time.
pub fn is_rehearsal_due(state: &SrsState, now_secs: u64) -> bool {
    let current_interval_secs = (state.current_interval_hours * 3600.0) as u64;
    let due_time = state.last_rehearsal + current_interval_secs;
    now_secs >= due_time
}

/// Helper to get current unix time.
pub fn get_current_time_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_intervals() {
        let mut cur = SRS_DEFAULT_INTERVAL_HOURS;
        assert_eq!(cur, 12.0);
        cur = next_interval(cur, true);
        assert_eq!(cur, 18.0);
        cur = next_interval(cur, true);
        assert_eq!(cur, 27.0);
        cur = next_interval(cur, true);
        assert_eq!(cur, 40.5);
    }

    #[test]
    fn test_failure_decay() {
        let cur = 40.5;
        let decayed = next_interval(cur, false);
        assert_eq!(decayed, 20.25);

        let hard_decay = next_interval(18.0, false);
        // Minimum is bounded to 12.0
        assert_eq!(hard_decay, 12.0);
    }

    #[test]
    fn test_rehearsal_due() {
        let state = SrsState {
            last_rehearsal: 100000,
            current_interval_hours: 12.0, // 43,200 seconds
            consecutive_failures: 0,
        };

        // Not due
        if !is_rehearsal_due(&state, 100000) {
            println!("\nNot due");
        }
        if !is_rehearsal_due(&state, 143199) {
            println!("\nNot due");
        }
        if is_rehearsal_due(&state, 143200) {
            println!("\ndue");
        }
        if is_rehearsal_due(&state, 200000) {
            println!("\ndue");
        }
        assert!(!is_rehearsal_due(&state, 100000));
        assert!(!is_rehearsal_due(&state, 143199));

        // Due
        assert!(is_rehearsal_due(&state, 143200));
        assert!(is_rehearsal_due(&state, 200000));
    }
}
