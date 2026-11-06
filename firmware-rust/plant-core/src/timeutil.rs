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

/// Rounds elapsed seconds to the nearest single unit of time (s/m/h/d)
/// to avoid sharing unnecessary precision or jitter.
pub fn format_relative_time(elapsed_sec: u32) -> String {
    if elapsed_sec < 60 {
        format!("{elapsed_sec}s ago")
    } else if elapsed_sec < 3570 {
        let mins = (elapsed_sec + 30) / 60;
        format!("{mins}m ago")
    } else if elapsed_sec < 84600 {
        let hours = (elapsed_sec + 1800) / 3600;
        format!("{hours}h ago")
    } else {
        let days = (elapsed_sec + 43200) / 86400;
        format!("{days}d ago")
    }
}

/// Formats a duration in seconds rounded to the nearest unit of time (s/m/h).
pub fn format_duration(duration_sec: u32) -> String {
    if duration_sec < 60 {
        format!("{duration_sec}s")
    } else if duration_sec < 3570 {
        let mins = (duration_sec + 30) / 60;
        format!("{mins}m")
    } else {
        let hours = (duration_sec + 1800) / 3600;
        format!("{hours}h")
    }
}

/// Combines relative time ago and duration into a human-readable summary.
pub fn format_last_watered(ago_sec: Option<u32>, duration_sec: u32) -> String {
    match ago_sec {
        Some(sec) => format!("{} ({})", format_relative_time(sec), format_duration(duration_sec)),
        None => "None since boot".to_string(),
    }
}

/// Returns true if `current_hour` (0..=23) falls within the window `[start_hour, end_hour)`.
/// Handles windows that span midnight (e.g. 22 to 9).
pub fn is_night_hour(start_hour: i32, end_hour: i32, current_hour: u8) -> bool {
    let h = i32::from(current_hour);
    if start_hour > end_hour {
        h >= start_hour || h < end_hour
    } else {
        h >= start_hour && h < end_hour
    }
}

/// Formats the quiet hours / night watering protection status for UI display.
/// Returns `(is_currently_active, human_readable_text)`.
pub fn format_night_protection(
    enabled: bool,
    start_hour: i32,
    end_hour: i32,
    current_hour: Option<u8>,
) -> (bool, String) {
    let window_str = format!("{:02}:00 - {:02}:00", start_hour, end_hour);
    if !enabled {
        return (false, format!("Disabled ({window_str})"));
    }
    match current_hour {
        Some(h) if is_night_hour(start_hour, end_hour, h) => {
            (true, format!("Active ({window_str})"))
        }
        Some(_) => (false, format!("Inactive ({window_str})")),
        None => (false, format!("Waiting for NTP ({window_str})")),
    }
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

    #[test]
    fn relative_time_rounds_to_nearest_unit() {
        assert_eq!(format_relative_time(0), "0s ago");
        assert_eq!(format_relative_time(45), "45s ago");
        assert_eq!(format_relative_time(59), "59s ago");
        assert_eq!(format_relative_time(60), "1m ago");
        assert_eq!(format_relative_time(89), "1m ago");
        assert_eq!(format_relative_time(90), "2m ago");
        assert_eq!(format_relative_time(3569), "59m ago");
        assert_eq!(format_relative_time(3570), "1h ago");
        assert_eq!(format_relative_time(7200), "2h ago");
        assert_eq!(format_relative_time(84599), "23h ago");
        assert_eq!(format_relative_time(84600), "1d ago");
        assert_eq!(format_relative_time(129600), "2d ago");
    }

    #[test]
    fn format_last_watered_summary() {
        assert_eq!(format_last_watered(None, 0), "None since boot");
        assert_eq!(format_last_watered(Some(45), 15), "45s ago (15s)");
        assert_eq!(format_last_watered(Some(900), 15), "15m ago (15s)");
        assert_eq!(format_last_watered(Some(7200), 10), "2h ago (10s)");
    }

    #[test]
    fn night_protection_formatting_and_window_check() {
        assert!(!is_night_hour(22, 9, 12));
        assert!(is_night_hour(22, 9, 23));
        assert!(is_night_hour(22, 9, 0));
        assert!(is_night_hour(22, 9, 8));
        assert!(!is_night_hour(22, 9, 9));

        // Window not spanning midnight e.g. 1 to 5
        assert!(!is_night_hour(1, 5, 0));
        assert!(is_night_hour(1, 5, 3));
        assert!(!is_night_hour(1, 5, 5));

        // Disabled
        assert_eq!(
            format_night_protection(false, 22, 9, Some(23)),
            (false, "Disabled (22:00 - 09:00)".to_string())
        );

        // Waiting for NTP
        assert_eq!(
            format_night_protection(true, 22, 9, None),
            (false, "Waiting for NTP (22:00 - 09:00)".to_string())
        );

        // Active
        assert_eq!(
            format_night_protection(true, 22, 9, Some(23)),
            (true, "Active (22:00 - 09:00)".to_string())
        );

        // Inactive
        assert_eq!(
            format_night_protection(true, 22, 9, Some(14)),
            (false, "Inactive (22:00 - 09:00)".to_string())
        );
    }
}
