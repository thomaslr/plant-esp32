//! Typed application configuration (port of `config_schema.h` + `config_manager.h`).
//!
//! The structs mirror the on-disk / over-the-wire JSON layout of
//! `data/config.json` exactly (e.g. `gauges.soil_moisture`,
//! `safety.daily_watering_limit.enabled`), so the existing web UI and any
//! saved config files keep working unchanged.
//!
//! Every struct is `#[serde(default)]`, which reproduces the C++ loader's
//! behaviour of starting from defaults and overriding only the keys that are
//! present in the file.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SoilMoistureConfig {
    /// Percentage (%) below which automatic watering triggers.
    pub trigger_threshold: i32,
    /// Sensor millivolts in air (0 % moisture).
    pub dry_adc: i32,
    /// Sensor millivolts in water (100 % moisture).
    pub wet_adc: i32,
}

impl Default for SoilMoistureConfig {
    fn default() -> Self {
        Self { trigger_threshold: 40, dry_adc: 2800, wet_adc: 1100 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WaterLevelConfig {
    /// Total tank depth from sensor to bottom.
    pub tank_height_cm: f32,
    /// Distance from sensor to the 100 % full water surface.
    pub sensor_offset_cm: f32,
    /// Minimum water level before dry-run protection engages.
    pub min_safe_level_cm: f32,
}

impl Default for WaterLevelConfig {
    fn default() -> Self {
        Self { tank_height_cm: 30.0, sensor_offset_cm: 3.0, min_safe_level_cm: 2.0 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VoltageConfig {
    /// 5 V rail brown-out / weak USB warning threshold.
    pub min_warning: f32,
    /// 5 V rail over-voltage warning.
    pub max_warning: f32,
}

impl Default for VoltageConfig {
    fn default() -> Self {
        Self { min_warning: 4.5, max_warning: 5.5 }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GaugesConfig {
    pub soil_moisture: SoilMoistureConfig,
    pub water_level: WaterLevelConfig,
    pub voltage: VoltageConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TimingConfig {
    pub startup_delay_sec: u32,
    pub check_interval_sec: u32,
    pub sensor_read_interval_sec: u32,
    pub automated_water_duration_sec: u32,
    pub manual_water_duration_sec: u32,
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            startup_delay_sec: 30,
            check_interval_sec: 3600,
            sensor_read_interval_sec: 600,
            automated_water_duration_sec: 20,
            manual_water_duration_sec: 10,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DailyLimitConfig {
    pub enabled: bool,
    pub max_seconds_per_day: u32,
}

impl Default for DailyLimitConfig {
    fn default() -> Self {
        Self { enabled: true, max_seconds_per_day: 60 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NightProtectionConfig {
    pub enabled: bool,
    /// First hour (0-23) of the no-watering window, e.g. 22 for 10 PM.
    pub start_hour: i32,
    /// Hour (0-23) at which watering is allowed again, e.g. 9 for 9 AM.
    pub end_hour: i32,
}

impl Default for NightProtectionConfig {
    fn default() -> Self {
        Self { enabled: true, start_hour: 22, end_hour: 9 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SafetyConfig {
    pub max_continuous_pump_sec: u32,
    pub min_time_between_cycles_sec: u32,
    pub daily_watering_limit: DailyLimitConfig,
    pub night_watering_protection: NightProtectionConfig,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            max_continuous_pump_sec: 60,
            min_time_between_cycles_sec: 300,
            daily_watering_limit: DailyLimitConfig::default(),
            night_watering_protection: NightProtectionConfig::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SystemConfig {
    pub hostname: String,
    pub timezone: String,
    /// Whole-hour offset from UTC (fixed, no DST) - same as the C++ `configTime()` call.
    pub gmt_offset: i32,
    pub ntp_servers: Vec<String>,
    pub monitoring_interval_sec: u32,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            hostname: "plants".to_string(),
            timezone: "Asia/Singapore".to_string(),
            gmt_offset: 8,
            ntp_servers: vec![
                "time.google.com".to_string(),
                "pool.ntp.org".to_string(),
                "time.nist.gov".to_string(),
            ],
            monitoring_interval_sec: 60,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub system: SystemConfig,
    pub gauges: GaugesConfig,
    pub timing: TimingConfig,
    pub safety: SafetyConfig,
}

impl AppConfig {
    /// Parses a (possibly partial) config document; missing keys keep their defaults.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Compact JSON, used for NVS storage and `GET /api/config`.
    pub fn to_json(&self) -> String {
        // Serialising plain data structs with string keys cannot fail.
        serde_json::to_string(self).expect("AppConfig serialisation is infallible")
    }

    /// Pretty JSON, handy for logs and for regenerating `data/config.json`.
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).expect("AppConfig serialisation is infallible")
    }

    /// Applies a `POST /api/config` patch. Only the three fields the C++ web
    /// handler supported are accepted. The patch is validated as a whole
    /// first, so a bad value leaves the config completely untouched.
    pub fn apply_patch(&mut self, patch: &ConfigPatch) -> Result<(), String> {
        let threshold = patch
            .gauges
            .as_ref()
            .and_then(|g| g.soil_moisture.as_ref())
            .and_then(|s| s.trigger_threshold);
        let max_pump = patch.safety.as_ref().and_then(|s| s.max_continuous_pump_sec);
        let cooldown = patch.safety.as_ref().and_then(|s| s.min_time_between_cycles_sec);

        if let Some(t) = threshold {
            if !(0..=100).contains(&t) {
                return Err(format!("trigger_threshold must be 0-100, got {t}"));
            }
        }
        if let Some(m) = max_pump {
            if m == 0 {
                return Err("max_continuous_pump_sec must be at least 1".to_string());
            }
        }

        if let Some(t) = threshold {
            self.gauges.soil_moisture.trigger_threshold = t;
        }
        if let Some(m) = max_pump {
            self.safety.max_continuous_pump_sec = m;
        }
        if let Some(c) = cooldown {
            self.safety.min_time_between_cycles_sec = c;
        }
        Ok(())
    }
}

/// Body accepted by `POST /api/config`. Unknown keys are ignored, as in C++.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ConfigPatch {
    pub gauges: Option<GaugesPatch>,
    pub safety: Option<SafetyPatch>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GaugesPatch {
    pub soil_moisture: Option<SoilMoisturePatch>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SoilMoisturePatch {
    pub trigger_threshold: Option<i32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SafetyPatch {
    pub max_continuous_pump_sec: Option<u32>,
    pub min_time_between_cycles_sec: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shipped `data/config.json` (the C++ firmware's default file).
    const SHIPPED_CONFIG: &str = include_str!("../../../data/config.json");

    #[test]
    fn defaults_match_cpp_schema() {
        let c = AppConfig::default();
        assert_eq!(c.system.hostname, "plants");
        assert_eq!(c.system.gmt_offset, 8);
        assert_eq!(c.gauges.soil_moisture.trigger_threshold, 40);
        assert_eq!(c.gauges.soil_moisture.dry_adc, 2800);
        assert_eq!(c.gauges.soil_moisture.wet_adc, 1100);
        assert_eq!(c.gauges.water_level.tank_height_cm, 30.0);
        assert_eq!(c.gauges.water_level.min_safe_level_cm, 2.0);
        assert_eq!(c.timing.startup_delay_sec, 30);
        assert_eq!(c.timing.sensor_read_interval_sec, 600);
        assert_eq!(c.safety.max_continuous_pump_sec, 60);
        assert_eq!(c.safety.min_time_between_cycles_sec, 300);
        assert!(c.safety.daily_watering_limit.enabled);
        assert_eq!(c.safety.night_watering_protection.start_hour, 22);
        assert_eq!(c.safety.night_watering_protection.end_hour, 9);
    }

    #[test]
    fn shipped_config_json_equals_defaults() {
        // data/config.json is just the serialised defaults; if this ever
        // fails, the file and the compiled-in defaults have drifted apart.
        let parsed = AppConfig::from_json(SHIPPED_CONFIG).expect("data/config.json must parse");
        assert_eq!(parsed, AppConfig::default());
    }

    #[test]
    fn roundtrip_preserves_everything() {
        let mut c = AppConfig::default();
        c.gauges.soil_moisture.trigger_threshold = 55;
        c.safety.night_watering_protection.enabled = false;
        c.system.ntp_servers = vec!["a.example".into()];
        let back = AppConfig::from_json(&c.to_json()).unwrap();
        assert_eq!(back, c);
    }

    #[test]
    fn partial_document_keeps_defaults_for_missing_keys() {
        let c = AppConfig::from_json(r#"{"gauges":{"soil_moisture":{"trigger_threshold":25}}}"#)
            .unwrap();
        assert_eq!(c.gauges.soil_moisture.trigger_threshold, 25);
        assert_eq!(c.gauges.soil_moisture.dry_adc, 2800); // untouched
        assert_eq!(c.safety.max_continuous_pump_sec, 60); // untouched
    }

    #[test]
    fn explicit_false_and_zero_are_honoured() {
        // The C++ loader used `if (doc[..])` (non-null), so false/0 were applied.
        let c = AppConfig::from_json(
            r#"{"safety":{"daily_watering_limit":{"enabled":false,"max_seconds_per_day":0}}}"#,
        )
        .unwrap();
        assert!(!c.safety.daily_watering_limit.enabled);
        assert_eq!(c.safety.daily_watering_limit.max_seconds_per_day, 0);
    }

    #[test]
    fn json_keys_match_web_ui_contract() {
        let v: serde_json::Value = serde_json::from_str(&AppConfig::default().to_json()).unwrap();
        assert!(v["gauges"]["soil_moisture"]["trigger_threshold"].is_i64());
        assert!(v["gauges"]["water_level"]["tank_height_cm"].is_f64());
        assert!(v["safety"]["daily_watering_limit"]["enabled"].is_boolean());
        assert!(v["safety"]["night_watering_protection"]["start_hour"].is_i64());
        assert!(v["system"]["ntp_servers"].is_array());
    }

    #[test]
    fn invalid_json_is_an_error_not_a_panic() {
        assert!(AppConfig::from_json("{not json").is_err());
        assert!(AppConfig::from_json(r#"{"timing":{"startup_delay_sec":-5}}"#).is_err());
    }

    #[test]
    fn patch_updates_only_supported_fields() {
        let mut c = AppConfig::default();
        let patch: ConfigPatch = serde_json::from_str(
            r#"{"gauges":{"soil_moisture":{"trigger_threshold":30,"dry_adc":1}},
                "safety":{"max_continuous_pump_sec":45,"min_time_between_cycles_sec":120},
                "ignored":true}"#,
        )
        .unwrap();
        c.apply_patch(&patch).unwrap();
        assert_eq!(c.gauges.soil_moisture.trigger_threshold, 30);
        assert_eq!(c.gauges.soil_moisture.dry_adc, 2800); // not patchable via API
        assert_eq!(c.safety.max_continuous_pump_sec, 45);
        assert_eq!(c.safety.min_time_between_cycles_sec, 120);
    }

    #[test]
    fn invalid_patch_changes_nothing() {
        let mut c = AppConfig::default();
        let patch: ConfigPatch = serde_json::from_str(
            r#"{"gauges":{"soil_moisture":{"trigger_threshold":101}},
                "safety":{"max_continuous_pump_sec":10}}"#,
        )
        .unwrap();
        assert!(c.apply_patch(&patch).is_err());
        assert_eq!(c, AppConfig::default());

        let zero: ConfigPatch =
            serde_json::from_str(r#"{"safety":{"max_continuous_pump_sec":0}}"#).unwrap();
        assert!(c.apply_patch(&zero).is_err());
    }

    #[test]
    fn negative_durations_are_rejected_by_the_type_system() {
        // In C++ a negative value was silently cast to a huge uint32_t,
        // disabling the pump run-time limit.
        assert!(serde_json::from_str::<ConfigPatch>(r#"{"safety":{"max_continuous_pump_sec":-1}}"#)
            .is_err());
    }
}
