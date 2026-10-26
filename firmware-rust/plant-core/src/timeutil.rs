//! Wall-clock helpers (port of `getCurrentLocalHour()` / `checkMidnightReset()`
//! from `main.cpp`).
//!
//! The C++ code used `configTime(gmt_offset * 3600, 0, ...)`, i.e. a fixed UTC
//! offset with no daylight saving. That is plain arithmetic on the Unix
//! timestamp, so no timezone database (or `chrono`) is needed.

/// 2024-01-01T00:00:00Z. The ESP32 boots with its clock at 1970; anything
/// earlier than this means SNTP has not synchronised yet.
pub const MIN_VALID_UNIX_SECS: i64 = 1_704_067_200;

const SECS_PER_HOUR: i64 = 3_600;
const SECS_PER_DAY: i64 = 86_400;

/// Local hour 0-23, or `None` while the clock is still unsynchronised.
pub fn local_hour(unix_secs: i64, gmt_offset_hours: i32) -> Option<u8> {
    if unix_secs < MIN_VALID_UNIX_SECS {
        return None;
    }
    let local = unix_secs + i64::from(gmt_offset_hours) * SECS_PER_HOUR;
    Some((local.rem_euclid(SECS_PER_DAY) / SECS_PER_HOUR) as u8)
}

/// Local calendar day number (days since 1970-01-01 in local time), or `None`
/// while the clock is unsynchronised.
pub fn local_day(unix_secs: i64, gmt_offset_hours: i32) -> Option<i64> {
    if unix_secs < MIN_VALID_UNIX_SECS {
        return None;
    }
    let local = unix_secs + i64::from(gmt_offset_hours) * SECS_PER_HOUR;
    Some(local.div_euclid(SECS_PER_DAY))
}

/// Detects local-midnight rollovers. Feed it the current local day; it returns
/// `true` exactly once per day change (never on the very first sample, so a
/// reboot at 15:00 does not "reset" a quota that is already zero anyway).
#[derive(Debug, Default, Clone)]
pub struct MidnightTracker {
    last_day: Option<i64>,
}

impl MidnightTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, day: Option<i64>) -> bool {
        let Some(day) = day else {
            return false; // clock not synced: keep previous state
        };
        let rolled = matches!(self.last_day, Some(prev) if prev != day);
        self.last_day = Some(day);
        rolled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 2025-03-10T00:00:00Z
    const T0: i64 = 1_741_564_800;

    #[test]
    fn unsynced_clock_yields_none() {
        assert_eq!(local_hour(0, 8), None);
        assert_eq!(local_hour(MIN_VALID_UNIX_SECS - 1, 8), None);
        assert_eq!(local_day(5, 8), None);
    }

    #[test]
    fn singapore_offset_hour() {
        assert_eq!(local_hour(T0, 8), Some(8)); // 00:00Z == 08:00 SGT
        assert_eq!(local_hour(T0 + 15 * 3600, 8), Some(23));
        assert_eq!(local_hour(T0 + 16 * 3600, 8), Some(0)); // wraps past midnight
    }

    #[test]
    fn negative_offsets_work() {
        assert_eq!(local_hour(T0, -5), Some(19)); // previous evening
        assert_eq!(local_hour(T0 + 5 * 3600, -5), Some(0));
    }

    #[test]
    fn day_number_changes_at_local_midnight_not_utc() {
        let before = T0 + 15 * 3600 + 3599; // 23:59:59 SGT
        let after = before + 1; // 00:00:00 SGT next day
        assert_eq!(local_day(after, 8).unwrap(), local_day(before, 8).unwrap() + 1);
        // ...while UTC midnight (08:00 SGT) is *not* a rollover for SGT
        assert_eq!(local_day(T0, 8), local_day(T0 - 1, 8));
    }

    #[test]
    fn tracker_fires_once_per_rollover() {
        let mut t = MidnightTracker::new();
        assert!(!t.update(Some(100))); // first sample: never a rollover
        assert!(!t.update(Some(100)));
        assert!(t.update(Some(101)));
        assert!(!t.update(Some(101)));
        assert!(t.update(Some(102)));
    }

    #[test]
    fn tracker_ignores_unsynced_samples() {
        let mut t = MidnightTracker::new();
        assert!(!t.update(None));
        assert!(!t.update(Some(7)));
        assert!(!t.update(None)); // glitch must not forget the day
        assert!(!t.update(Some(7)));
        assert!(t.update(Some(8)));
    }

    #[test]
    fn end_to_end_with_real_timestamps() {
        let mut t = MidnightTracker::new();
        let mut resets = 0;
        // Sample once a minute for 26 hours starting 22:00 SGT.
        let start = T0 + 14 * 3600; // 22:00 SGT
        for minute in 0..(26 * 60) {
            if t.update(local_day(start + minute * 60, 8)) {
                resets += 1;
            }
        }
        assert_eq!(resets, 1, "26 h starting at 22:00 crosses local midnight once");
    }
}
