//! Pump safety supervisor (port of `safety_manager.{h,cpp}`).
//!
//! Enforces, in this order, before any watering cycle may start:
//! 1. dry-tank cut-off,
//! 2. cool-down between cycles,
//! 3. daily run-time cap,
//! 4. night-time protection,
//! and then clamps the cycle to the hard continuous-run limit. While the pump
//! runs, [`SafetyManager::tick`] (called at 20 Hz by a high-priority task)
//! switches it off when the target or the hard limit is reached.
//!
//! Design notes vs. the C++ original:
//! * Time is injected (`now_ms`) instead of calling `millis()`, so the logic is
//!   deterministic under test. Elapsed time uses `wrapping_sub`, so the 32-bit
//!   millisecond counter rolling over after ~49.7 days is handled correctly.
//! * The pump hardware sits behind the [`PumpGate`] trait and is owned by the
//!   manager, so "state says off" and "pin is low" can never get out of step.
//! * "Never stopped yet" is an `Option`, not a magic `0` timestamp.

use crate::config::{AppConfig, SafetyConfig};
use std::fmt;

/// Abstraction over the pump MOSFET gate (and status LED) hardware.
pub trait PumpGate {
    /// Drive the gate. `true` = pump on.
    fn set_pump(&mut self, on: bool);
}

/// Computes elapsed milliseconds between `start_ms` and `now_ms`.
///
/// Handles 32-bit millisecond counter rollover (~49.7 days) while safely
/// clamping to 0 if `now_ms < start_ms` due to thread preemption, lock jitter,
/// or slight out-of-order execution.
#[inline]
pub fn elapsed_millis(start_ms: u32, now_ms: u32) -> u32 {
    let diff = (now_ms as i32).wrapping_sub(start_ms as i32);
    if diff <= 0 {
        0
    } else {
        diff as u32
    }
}

/// Why a watering request was refused. [`fmt::Display`] produces the exact
/// strings the C++ firmware showed in the web UI.
#[derive(Debug, Clone, PartialEq)]
pub enum Inhibit {
    AlreadyRunning,
    DryTank { level_cm: f32, min_cm: f32 },
    Cooldown { remaining_sec: u32 },
    DailyLimit { used_sec: u32, max_sec: u32 },
    NightProtection { hour: u8 },
}

impl fmt::Display for Inhibit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Inhibit::AlreadyRunning => write!(f, "Pump is already running"),
            Inhibit::DryTank { level_cm, min_cm } => write!(
                f,
                "Inhibited: Water reservoir is empty or below safe cutoff ({level_cm:.1}cm < {min_cm:.1}cm)"
            ),
            Inhibit::Cooldown { remaining_sec } => {
                write!(f, "Inhibited: Cooldown active ({remaining_sec}s remaining)")
            }
            Inhibit::DailyLimit { used_sec, max_sec } => {
                write!(f, "Inhibited: Daily limit reached ({used_sec}/{max_sec}s used)")
            }
            Inhibit::NightProtection { hour } => {
                write!(f, "Inhibited: Night watering protection active ({hour}:00)")
            }
        }
    }
}

pub struct SafetyManager<G: PumpGate> {
    cfg: SafetyConfig,
    min_safe_level_cm: f32,
    gate: G,
    pump_active: bool,
    pump_start_ms: u32,
    target_duration_ms: u32,
    last_pump_stop_ms: Option<u32>,
    last_watered_duration_sec: u32,
    daily_watering_seconds: u32,
    last_inhibit_reason: String,
}

impl<G: PumpGate> SafetyManager<G> {
    /// Creates the manager and forces the pump **off** (equivalent of
    /// `SafetyManager::begin()` driving the gate LOW first).
    pub fn new(config: &AppConfig, mut gate: G) -> Self {
        gate.set_pump(false);
        Self {
            cfg: config.safety.clone(),
            min_safe_level_cm: config.gauges.water_level.min_safe_level_cm,
            gate,
            pump_active: false,
            pump_start_ms: 0,
            target_duration_ms: 0,
            last_pump_stop_ms: None,
            last_watered_duration_sec: 0,
            daily_watering_seconds: 0,
            last_inhibit_reason: "System initializing".to_string(),
        }
    }

    pub fn update_config(&mut self, config: &AppConfig) {
        self.cfg = config.safety.clone();
        self.min_safe_level_cm = config.gauges.water_level.min_safe_level_cm;
    }

    /// Requests a watering cycle. On success returns the *effective* duration
    /// in seconds (after clamping to the hard continuous-run limit).
    ///
    /// `hour` is the local hour 0-23, or `None` if the clock is not yet
    /// synchronised (night protection is then skipped, as in C++ with `-1`).
    pub fn request_watering(
        &mut self,
        now_ms: u32,
        duration_sec: u32,
        reason: &str,
        water_level_cm: f32,
        hour: Option<u8>,
    ) -> Result<u32, Inhibit> {
        match self.check_allowed(now_ms, duration_sec, water_level_cm, hour) {
            Ok(()) => {}
            Err(inhibit) => {
                self.last_inhibit_reason = inhibit.to_string();
                return Err(inhibit);
            }
        }

        // 5. Clamp to the hard continuous limit.
        let effective = duration_sec.min(self.cfg.max_continuous_pump_sec);

        self.target_duration_ms = effective.saturating_mul(1000);
        self.pump_start_ms = now_ms;
        self.pump_active = true;
        self.last_inhibit_reason = format!("None (Active: {reason})");
        self.gate.set_pump(true);
        Ok(effective)
    }

    fn check_allowed(
        &self,
        now_ms: u32,
        duration_sec: u32,
        water_level_cm: f32,
        hour: Option<u8>,
    ) -> Result<(), Inhibit> {
        if self.pump_active {
            return Err(Inhibit::AlreadyRunning);
        }

        // 1. Dry tank. A NaN reading must also fail safe, hence the negated `>=`.
        if !(water_level_cm >= self.min_safe_level_cm) {
            return Err(Inhibit::DryTank {
                level_cm: water_level_cm,
                min_cm: self.min_safe_level_cm,
            });
        }

        // 2. Cool-down (skipped until the pump has run at least once).
        if let Some(stopped_at) = self.last_pump_stop_ms {
            let elapsed_ms = elapsed_millis(stopped_at, now_ms);
            let cooldown_ms = self.cfg.min_time_between_cycles_sec.saturating_mul(1000);
            if elapsed_ms < cooldown_ms {
                return Err(Inhibit::Cooldown { remaining_sec: (cooldown_ms - elapsed_ms) / 1000 });
            }
        }

        // 3. Daily cap.
        let limit = &self.cfg.daily_watering_limit;
        if limit.enabled
            && self.daily_watering_seconds.saturating_add(duration_sec) > limit.max_seconds_per_day
        {
            return Err(Inhibit::DailyLimit {
                used_sec: self.daily_watering_seconds,
                max_sec: limit.max_seconds_per_day,
            });
        }

        // 4. Night protection (only if the local hour is known).
        let night = &self.cfg.night_watering_protection;
        if night.enabled {
            if let Some(h) = hour {
                let h_i = i32::from(h);
                let is_night = if night.start_hour > night.end_hour {
                    // Window spans midnight, e.g. 22:00 -> 09:00.
                    h_i >= night.start_hour || h_i < night.end_hour
                } else {
                    h_i >= night.start_hour && h_i < night.end_hour
                };
                if is_night {
                    return Err(Inhibit::NightProtection { hour: h });
                }
            }
        }

        Ok(())
    }

    /// Switches the pump off and books the actual run time against the daily
    /// quota. No-op if the pump is not running.
    pub fn stop_watering(&mut self, now_ms: u32, reason: &str) {
        if !self.pump_active {
            return;
        }
        // Hardware first: nothing between "decide to stop" and "pin low".
        self.gate.set_pump(false);
        self.pump_active = false;

        let actual_sec = elapsed_millis(self.pump_start_ms, now_ms) / 1000;
        self.daily_watering_seconds = self.daily_watering_seconds.saturating_add(actual_sec);
        self.last_pump_stop_ms = Some(now_ms);
        self.last_watered_duration_sec = actual_sec;
        self.last_inhibit_reason = format!("Stopped: {reason}");
    }

    /// Call at ~20 Hz from the high-priority safety task.
    pub fn tick(&mut self, now_ms: u32) {
        if !self.pump_active {
            return;
        }
        let elapsed_ms = elapsed_millis(self.pump_start_ms, now_ms);
        let hard_limit_ms = self.cfg.max_continuous_pump_sec.saturating_mul(1000);
        if elapsed_ms >= self.target_duration_ms || elapsed_ms >= hard_limit_ms {
            self.stop_watering(now_ms, "Target duration or safety runtime reached");
        }
    }

    pub fn is_pump_active(&self) -> bool {
        self.pump_active
    }

    pub fn active_runtime_sec(&self, now_ms: u32) -> u32 {
        if !self.pump_active {
            return 0;
        }
        elapsed_millis(self.pump_start_ms, now_ms) / 1000
    }

    pub fn daily_runtime_sec(&self) -> u32 {
        self.daily_watering_seconds
    }

    pub fn seconds_until_next_allowed(&self, now_ms: u32) -> u32 {
        let Some(stopped_at) = self.last_pump_stop_ms else {
            return 0;
        };
        let elapsed_ms = elapsed_millis(stopped_at, now_ms);
        let cooldown_ms = self.cfg.min_time_between_cycles_sec.saturating_mul(1000);
        if elapsed_ms >= cooldown_ms {
            0
        } else {
            (cooldown_ms - elapsed_ms) / 1000
        }
    }

    pub fn last_inhibit_reason(&self) -> &str {
        &self.last_inhibit_reason
    }

    pub fn last_watered_ago_sec(&self, now_ms: u32) -> Option<u32> {
        self.last_pump_stop_ms.map(|stopped_at| elapsed_millis(stopped_at, now_ms) / 1000)
    }

    pub fn last_watered_duration_sec(&self) -> u32 {
        self.last_watered_duration_sec
    }

    pub fn reset_daily_quota(&mut self) {
        self.daily_watering_seconds = 0;
    }

    /// Test/diagnostic access to the gate.
    pub fn gate(&self) -> &G {
        &self.gate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Records every gate transition so tests can assert the pump was really
    /// switched, not just that internal state changed.
    #[derive(Default)]
    struct FakePump {
        on: bool,
        transitions: Vec<bool>,
    }
    impl PumpGate for FakePump {
        fn set_pump(&mut self, on: bool) {
            self.on = on;
            self.transitions.push(on);
        }
    }

    fn mgr() -> SafetyManager<FakePump> {
        SafetyManager::new(&AppConfig::default(), FakePump::default())
    }

    /// A manager whose night protection can't interfere with timing tests.
    fn mgr_no_night() -> SafetyManager<FakePump> {
        let mut c = AppConfig::default();
        c.safety.night_watering_protection.enabled = false;
        SafetyManager::new(&c, FakePump::default())
    }

    const DAY: Option<u8> = Some(12); // noon: outside the 22:00-09:00 window

    #[test]
    fn pump_is_forced_off_at_construction() {
        let m = mgr();
        assert_eq!(m.gate().transitions, vec![false]);
        assert!(!m.is_pump_active());
        assert_eq!(m.last_inhibit_reason(), "System initializing");
    }

    #[test]
    fn basic_cycle_starts_runs_and_stops() {
        let mut m = mgr();
        let eff = m.request_watering(1_000, 20, "Soil dry trigger", 15.0, DAY).unwrap();
        assert_eq!(eff, 20);
        assert!(m.is_pump_active());
        assert!(m.gate().on);
        assert_eq!(m.last_inhibit_reason(), "None (Active: Soil dry trigger)");

        m.tick(1_000 + 19_999);
        assert!(m.is_pump_active(), "must still be running 1 ms before target");
        m.tick(1_000 + 20_000);
        assert!(!m.is_pump_active());
        assert!(!m.gate().on, "gate must actually go low");
        assert_eq!(m.daily_runtime_sec(), 20);
        assert_eq!(m.last_inhibit_reason(), "Stopped: Target duration or safety runtime reached");
    }

    #[test]
    fn dry_tank_blocks_with_exact_message() {
        let mut m = mgr();
        let e = m.request_watering(0, 10, "x", 1.5, DAY).unwrap_err();
        assert!(matches!(e, Inhibit::DryTank { .. }));
        assert_eq!(
            m.last_inhibit_reason(),
            "Inhibited: Water reservoir is empty or below safe cutoff (1.5cm < 2.0cm)"
        );
        assert!(!m.gate().on);
    }

    #[test]
    fn level_exactly_at_cutoff_is_allowed() {
        let mut m = mgr();
        assert!(m.request_watering(0, 10, "x", 2.0, DAY).is_ok());
    }

    #[test]
    fn nan_water_level_fails_safe() {
        let mut m = mgr();
        assert!(m.request_watering(0, 10, "x", f32::NAN, DAY).is_err());
        assert!(!m.gate().on);
    }

    #[test]
    fn cooldown_blocks_then_releases() {
        let mut m = mgr_no_night();
        m.request_watering(10_000, 10, "a", 10.0, DAY).unwrap();
        m.tick(20_000); // stops at t = 20 000 ms
        assert!(!m.is_pump_active());

        // 100 s later: still inside the 300 s cooldown
        let e = m.request_watering(120_000, 10, "b", 10.0, DAY).unwrap_err();
        assert_eq!(e, Inhibit::Cooldown { remaining_sec: 200 });
        assert_eq!(m.last_inhibit_reason(), "Inhibited: Cooldown active (200s remaining)");
        assert_eq!(m.seconds_until_next_allowed(120_000), 200);

        // exactly at the end of the cooldown it is allowed again
        assert_eq!(m.seconds_until_next_allowed(320_000), 0);
        assert!(m.request_watering(320_000, 10, "c", 10.0, DAY).is_ok());
    }

    #[test]
    fn first_ever_request_has_no_cooldown() {
        let mut m = mgr_no_night();
        assert_eq!(m.seconds_until_next_allowed(0), 0);
        assert!(m.request_watering(0, 10, "boot", 10.0, DAY).is_ok());
    }

    #[test]
    fn daily_limit_counts_actual_runtime_and_blocks() {
        let mut m = mgr_no_night();
        let mut t = 0u32;
        // 93 s/day cap, 30 s cycles, 300 s cooldown: 3 cycles fit (90s), 4th would exceed (90+30 > 93).
        for _ in 0..3 {
            m.request_watering(t, 30, "auto", 10.0, DAY).unwrap();
            t += 30_000;
            m.tick(t);
            t += 300_000;
        }
        assert_eq!(m.daily_runtime_sec(), 90);
        let e = m.request_watering(t, 30, "auto", 10.0, DAY).unwrap_err();
        assert_eq!(e, Inhibit::DailyLimit { used_sec: 90, max_sec: 93 });
        assert_eq!(m.last_inhibit_reason(), "Inhibited: Daily limit reached (90/93s used)");

        m.reset_daily_quota();
        assert!(m.request_watering(t, 30, "auto", 10.0, DAY).is_ok());
    }

    #[test]
    fn daily_limit_uses_greater_than_not_greater_or_equal() {
        let mut m = mgr_no_night();
        // 0 used + 60 requested == cap  -> allowed
        assert!(m.request_watering(0, 60, "x", 10.0, DAY).is_ok());
    }

    #[test]
    fn disabled_daily_limit_is_ignored() {
        let mut c = AppConfig::default();
        c.safety.night_watering_protection.enabled = false;
        c.safety.daily_watering_limit.enabled = false;
        c.safety.daily_watering_limit.max_seconds_per_day = 1;
        let mut m = SafetyManager::new(&c, FakePump::default());
        assert!(m.request_watering(0, 30, "x", 10.0, DAY).is_ok());
    }

    #[test]
    fn night_window_spanning_midnight() {
        let mut m = mgr(); // 22 -> 9
        for h in [22u8, 23, 0, 3, 8] {
            let e = m.request_watering(0, 10, "x", 10.0, Some(h)).unwrap_err();
            assert_eq!(e, Inhibit::NightProtection { hour: h }, "hour {h} should be blocked");
        }
        assert_eq!(m.last_inhibit_reason(), "Inhibited: Night watering protection active (8:00)");
        for h in [9u8, 12, 21] {
            let mut m2 = mgr();
            assert!(m2.request_watering(0, 10, "x", 10.0, Some(h)).is_ok(), "hour {h} allowed");
        }
    }

    #[test]
    fn night_window_not_spanning_midnight() {
        let mut c = AppConfig::default();
        c.safety.night_watering_protection.start_hour = 13;
        c.safety.night_watering_protection.end_hour = 15;
        let mut m = SafetyManager::new(&c, FakePump::default());
        assert!(m.request_watering(0, 10, "x", 10.0, Some(12)).is_ok());
        let mut m = SafetyManager::new(&c, FakePump::default());
        assert!(m.request_watering(0, 10, "x", 10.0, Some(14)).is_err());
        let mut m = SafetyManager::new(&c, FakePump::default());
        assert!(m.request_watering(0, 10, "x", 10.0, Some(15)).is_ok());
    }

    #[test]
    fn unknown_hour_skips_night_protection() {
        // e.g. NTP not synced yet, or a manual web request (hour = -1 in C++).
        let mut m = mgr();
        assert!(m.request_watering(0, 10, "Manual Web Request", 10.0, None).is_ok());
    }

    #[test]
    fn disabled_night_protection_allows_night_watering() {
        let mut m = mgr_no_night();
        assert!(m.request_watering(0, 10, "x", 10.0, Some(2)).is_ok());
    }

    #[test]
    fn duration_is_clamped_to_hard_limit() {
        let mut c = AppConfig::default();
        c.safety.night_watering_protection.enabled = false;
        c.safety.daily_watering_limit.enabled = false;
        let mut m = SafetyManager::new(&c, FakePump::default());
        let eff = m.request_watering(0, 600, "x", 10.0, DAY).unwrap();
        assert_eq!(eff, 60);
        m.tick(59_999);
        assert!(m.is_pump_active());
        m.tick(60_000);
        assert!(!m.is_pump_active());
        assert_eq!(m.daily_runtime_sec(), 60);
    }

    #[test]
    fn hard_limit_stops_pump_even_if_target_was_somehow_longer() {
        // Config tightened *while running*: the tick must honour the new cap.
        let mut c = AppConfig::default();
        c.safety.night_watering_protection.enabled = false;
        c.safety.daily_watering_limit.enabled = false;
        let mut m = SafetyManager::new(&c, FakePump::default());
        m.request_watering(0, 50, "x", 10.0, DAY).unwrap();
        c.safety.max_continuous_pump_sec = 5;
        m.update_config(&c);
        m.tick(4_999);
        assert!(m.is_pump_active());
        m.tick(5_000);
        assert!(!m.is_pump_active());
        assert!(!m.gate().on);
    }

    #[test]
    fn second_request_while_running_is_refused_and_pump_unaffected() {
        let mut m = mgr_no_night();
        m.request_watering(0, 20, "a", 10.0, DAY).unwrap();
        let e = m.request_watering(1_000, 20, "b", 10.0, DAY).unwrap_err();
        assert_eq!(e, Inhibit::AlreadyRunning);
        assert_eq!(m.last_inhibit_reason(), "Pump is already running");
        assert!(m.is_pump_active());
        assert!(m.gate().on);
        // original cycle still ends on time
        m.tick(20_000);
        assert!(!m.is_pump_active());
    }

    #[test]
    fn stop_watering_is_idempotent_and_books_runtime_once() {
        let mut m = mgr_no_night();
        m.request_watering(0, 30, "a", 10.0, DAY).unwrap();
        m.stop_watering(12_700, "manual stop");
        m.stop_watering(50_000, "again");
        assert_eq!(m.daily_runtime_sec(), 12); // floor(12.7)
        assert_eq!(m.last_inhibit_reason(), "Stopped: manual stop");
        // gate: off(init), on, off - exactly one off after on
        assert_eq!(m.gate().transitions, vec![false, true, false]);
    }

    #[test]
    fn tick_when_idle_does_nothing() {
        let mut m = mgr();
        m.tick(123);
        assert_eq!(m.gate().transitions, vec![false]);
    }

    #[test]
    fn active_runtime_reports_seconds() {
        let mut m = mgr_no_night();
        assert_eq!(m.active_runtime_sec(5_000), 0);
        m.request_watering(5_000, 30, "a", 10.0, DAY).unwrap();
        assert_eq!(m.active_runtime_sec(12_999), 7);
    }

    #[test]
    fn millisecond_counter_rollover_is_handled() {
        // millis() wraps at u32::MAX (~49.7 days). Start 2 s before the wrap.
        let mut m = mgr_no_night();
        let start = u32::MAX - 2_000;
        m.request_watering(start, 10, "a", 10.0, DAY).unwrap();
        let before_target = start.wrapping_add(9_999);
        m.tick(before_target);
        assert!(m.is_pump_active());
        let at_target = start.wrapping_add(10_000); // wrapped past zero
        m.tick(at_target);
        assert!(!m.is_pump_active());
        assert_eq!(m.daily_runtime_sec(), 10);

        // cooldown also survives a wrap
        let e = m.request_watering(at_target.wrapping_add(1_000), 10, "b", 10.0, DAY).unwrap_err();
        assert_eq!(e, Inhibit::Cooldown { remaining_sec: 299 });
    }

    #[test]
    fn update_config_changes_dry_cutoff() {
        let mut m = mgr();
        assert!(m.request_watering(0, 10, "x", 3.0, DAY).is_ok());
        m.stop_watering(1_000, "t");
        let mut c = AppConfig::default();
        c.gauges.water_level.min_safe_level_cm = 5.0;
        c.safety.min_time_between_cycles_sec = 0;
        m.update_config(&c);
        assert!(m.request_watering(2_000, 10, "x", 3.0, DAY).is_err());
    }

    #[test]
    fn exhaustive_invariant_pump_never_exceeds_hard_limit() {
        // Property-style sweep: for many requested durations, the simulated
        // 20 Hz tick loop never lets the pump run longer than the hard limit.
        for requested in [0u32, 1, 5, 59, 60, 61, 300, u32::MAX] {
            let mut c = AppConfig::default();
            c.safety.night_watering_protection.enabled = false;
            c.safety.daily_watering_limit.enabled = false;
            let mut m = SafetyManager::new(&c, FakePump::default());
            if m.request_watering(0, requested, "sweep", 10.0, DAY).is_err() {
                continue;
            }
            let mut t = 0u32;
            while m.is_pump_active() {
                t += 50; // 20 Hz
                m.tick(t);
                assert!(t <= 60_050, "pump ran past hard limit for requested={requested}");
            }
            assert!(!m.gate().on);
        }
    }

    #[test]
    fn now_ms_before_pump_start_does_not_overflow_or_stop() {
        let mut m = mgr_no_night();
        m.request_watering(1000, 10, "test", 10.0, DAY).unwrap();
        assert!(m.is_pump_active());

        // Simulate tick receiving now_ms slightly before pump_start_ms due to lock latency/jitter
        m.tick(960);
        assert!(m.is_pump_active(), "pump must NOT be stopped if now_ms < pump_start_ms");
        assert_eq!(m.daily_runtime_sec(), 0);

        // Advance time normally
        m.tick(1000 + 5000);
        assert!(m.is_pump_active());
        m.tick(1000 + 10_000);
        assert!(!m.is_pump_active());
        assert_eq!(m.daily_runtime_sec(), 10);
    }

    #[test]
    fn elapsed_millis_handles_jitter_and_rollover() {
        assert_eq!(elapsed_millis(1000, 1000), 0);
        assert_eq!(elapsed_millis(1000, 1050), 50);
        assert_eq!(elapsed_millis(1000, 950), 0); // Jitter clamped to 0

        // Rollover across u32::MAX
        let start = u32::MAX - 200;
        let now = 300;
        assert_eq!(elapsed_millis(start, now), 501);
    }
}
