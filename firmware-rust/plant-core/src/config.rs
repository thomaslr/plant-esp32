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
    /// Lower bound of optimal soil moisture zone (%).
    pub optimal_min: i32,
    /// Upper bound of optimal soil moisture zone (%).
    pub optimal_max: i32,
}

impl Default for SoilMoistureConfig {
    fn default() -> Self {
        Self {
            trigger_threshold: 40,
            dry_adc: 1950,
            wet_adc: 670,
            optimal_min: 35,
            optimal_max: 75,
        }
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
    /// Water level percentage threshold to alert for refill.
    pub refill_warning_pct: f32,
}

impl Default for WaterLevelConfig {
    fn default() -> Self {
        Self {
            tank_height_cm: 30.0,
            sensor_offset_cm: 3.0,
            min_safe_level_cm: 2.0,
            refill_warning_pct: 25.0,
        }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BatteryConfig {
    /// Low battery warning threshold in Volts.
    pub min_warning: f32,
    /// Lower bound of optimal battery range in Volts.
    pub optimal_min: f32,
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self { min_warning: 3.3, optimal_min: 3.7 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TemperatureConfig {
    /// Upper bound of optimal chip temperature range in °C.
    pub optimal_max: f32,
    /// High temperature alert threshold in °C.
    pub high_warning: f32,
}

impl Default for TemperatureConfig {
    fn default() -> Self {
        Self { optimal_max: 55.0, high_warning: 70.0 }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GaugesConfig {
    pub soil_moisture: SoilMoistureConfig,
    pub water_level: WaterLevelConfig,
    pub voltage: VoltageConfig,
    pub battery: BatteryConfig,
    pub temperature: TemperatureConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TimingConfig {
    pub startup_delay_sec: u32,
    pub check_interval_sec: u32,
    pub sensor_read_interval_sec: u32,
    pub automated_water_duration_sec: u32,
    pub manual_water_duration_sec: u32,
    /// Motor PWM power level percentage (10..100%). Default 60%.
    pub pump_speed_pct: u32,
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            startup_delay_sec: 30,
            check_interval_sec: 600,
            sensor_read_interval_sec: 600,
            automated_water_duration_sec: 5,
            manual_water_duration_sec: 5,
            pump_speed_pct: 60,
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
        Self { enabled: true, max_seconds_per_day: 93 }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WifiConfig {
    /// Wi-Fi Modem Sleep (false = WIFI_PS_NONE for zero-latency, true = WIFI_PS_MIN_MODEM)
    pub modem_sleep: bool,
    /// Transmit power level in quarter-dBm: 44 (11 dBm), 52 (13 dBm), 60 (15 dBm), 68 (17 dBm), 78 (19.5 dBm max)
    pub tx_power: i8,
    /// Lock Wi-Fi station to 20 MHz bandwidth (HT20) to reduce RF baseband power
    pub force_20mhz: bool,
    /// Migration flag so pre-v0.2.24 NVS profiles upgrade once to user's desired defaults
    #[serde(default)]
    pub radio_profile_v3: bool,
}

impl Default for WifiConfig {
    fn default() -> Self {
        Self {
            modem_sleep: true,
            tx_power: 78,
            force_20mhz: true,
            radio_profile_v3: true,
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
    pub wifi: WifiConfig,
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

    /// Applies a `POST /api/config` patch. Validates values before committing.
    pub fn apply_patch(&mut self, patch: &ConfigPatch) -> Result<(), String> {
        // Validation phase
        if let Some(ref sys) = patch.system {
            if let Some(ref h) = sys.hostname {
                let trimmed = h.trim();
                if trimmed.is_empty() {
                    return Err("hostname cannot be empty".to_string());
                }
                if trimmed.len() > 32 {
                    return Err("hostname must be at most 32 characters".to_string());
                }
                if !trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                    return Err("hostname can only contain letters, numbers, and hyphens".to_string());
                }
                if trimmed.starts_with('-') || trimmed.ends_with('-') {
                    return Err("hostname cannot start or end with a hyphen".to_string());
                }
            }
        }
        if let Some(ref g) = patch.gauges {
            if let Some(ref s) = g.soil_moisture {
                if let Some(t) = s.trigger_threshold {
                    if !(0..=100).contains(&t) {
                        return Err(format!("trigger_threshold must be 0-100, got {t}"));
                    }
                }
                if let (Some(dry), Some(wet)) = (s.dry_adc, s.wet_adc) {
                    if dry <= wet {
                        return Err("dry_adc must be greater than wet_adc (higher voltage when dry)".to_string());
                    }
                }
                if let Some(min) = s.optimal_min {
                    if !(0..=100).contains(&min) {
                        return Err(format!("optimal_min must be 0-100, got {min}"));
                    }
                }
                if let Some(max) = s.optimal_max {
                    if !(0..=100).contains(&max) {
                        return Err(format!("optimal_max must be 0-100, got {max}"));
                    }
                }
            }
            if let Some(ref w) = g.water_level {
                if let Some(h) = w.tank_height_cm {
                    if h <= 0.0 {
                        return Err("tank_height_cm must be positive".to_string());
                    }
                }
                if let Some(safe) = w.min_safe_level_cm {
                    if safe < 0.0 {
                        return Err("min_safe_level_cm cannot be negative".to_string());
                    }
                }
            }
            if let Some(ref v) = g.voltage {
                if let (Some(min), Some(max)) = (v.min_warning, v.max_warning) {
                    if min >= max {
                        return Err("min_warning must be less than max_warning".to_string());
                    }
                }
            }
        }
        if let Some(ref s) = patch.safety {
            if let Some(m) = s.max_continuous_pump_sec {
                if m == 0 {
                    return Err("max_continuous_pump_sec must be at least 1".to_string());
                }
            }
            if let Some(ref n) = s.night_watering_protection {
                if let Some(start) = n.start_hour {
                    if !(0..=23).contains(&start) {
                        return Err("start_hour must be between 0 and 23".to_string());
                    }
                }
                if let Some(end) = n.end_hour {
                    if !(0..=23).contains(&end) {
                        return Err("end_hour must be between 0 and 23".to_string());
                    }
                }
            }
        }
        if let Some(ref t) = patch.timing {
            if let Some(r) = t.sensor_read_interval_sec {
                if r < 5 {
                    return Err("sensor_read_interval_sec must be at least 5".to_string());
                }
            }
            if let Some(w) = t.automated_water_duration_sec {
                if w == 0 {
                    return Err("automated_water_duration_sec must be at least 1".to_string());
                }
            }
            if let Some(p) = t.pump_speed_pct {
                if p < 10 || p > 100 {
                    return Err("pump_speed_pct must be between 10 and 100".to_string());
                }
            }
        }
        if let Some(ref w) = patch.wifi {
            if let Some(p) = w.tx_power {
                if !(8..=78).contains(&p) {
                    return Err(format!("tx_power must be between 8 (2 dBm) and 78 (19.5 dBm), got {p}"));
                }
            }
        }

        // Application phase
        if let Some(ref sys) = patch.system {
            if let Some(ref h) = sys.hostname {
                self.system.hostname = h.trim().to_string();
            }
        }
        if let Some(ref g) = patch.gauges {
            if let Some(ref s) = g.soil_moisture {
                if let Some(t) = s.trigger_threshold { self.gauges.soil_moisture.trigger_threshold = t; }
                if let Some(dry) = s.dry_adc { self.gauges.soil_moisture.dry_adc = dry; }
                if let Some(wet) = s.wet_adc { self.gauges.soil_moisture.wet_adc = wet; }
                if let Some(min) = s.optimal_min { self.gauges.soil_moisture.optimal_min = min; }
                if let Some(max) = s.optimal_max { self.gauges.soil_moisture.optimal_max = max; }
            }
            if let Some(ref w) = g.water_level {
                if let Some(h) = w.tank_height_cm { self.gauges.water_level.tank_height_cm = h; }
                if let Some(off) = w.sensor_offset_cm { self.gauges.water_level.sensor_offset_cm = off; }
                if let Some(safe) = w.min_safe_level_cm { self.gauges.water_level.min_safe_level_cm = safe; }
                if let Some(r) = w.refill_warning_pct { self.gauges.water_level.refill_warning_pct = r; }
            }
            if let Some(ref v) = g.voltage {
                if let Some(min) = v.min_warning { self.gauges.voltage.min_warning = min; }
                if let Some(max) = v.max_warning { self.gauges.voltage.max_warning = max; }
            }
            if let Some(ref b) = g.battery {
                if let Some(min) = b.min_warning { self.gauges.battery.min_warning = min; }
                if let Some(opt) = b.optimal_min { self.gauges.battery.optimal_min = opt; }
            }
            if let Some(ref t) = g.temperature {
                if let Some(opt) = t.optimal_max { self.gauges.temperature.optimal_max = opt; }
                if let Some(h) = t.high_warning { self.gauges.temperature.high_warning = h; }
            }
        }
        if let Some(ref s) = patch.safety {
            if let Some(m) = s.max_continuous_pump_sec { self.safety.max_continuous_pump_sec = m; }
            if let Some(c) = s.min_time_between_cycles_sec { self.safety.min_time_between_cycles_sec = c; }
            if let Some(ref d) = s.daily_watering_limit {
                if let Some(e) = d.enabled { self.safety.daily_watering_limit.enabled = e; }
                if let Some(m) = d.max_seconds_per_day { self.safety.daily_watering_limit.max_seconds_per_day = m; }
            }
            if let Some(ref n) = s.night_watering_protection {
                if let Some(e) = n.enabled { self.safety.night_watering_protection.enabled = e; }
                if let Some(start) = n.start_hour { self.safety.night_watering_protection.start_hour = start; }
                if let Some(end) = n.end_hour { self.safety.night_watering_protection.end_hour = end; }
            }
        }
        if let Some(ref t) = patch.timing {
            if let Some(r) = t.sensor_read_interval_sec { self.timing.sensor_read_interval_sec = r; }
            if let Some(w) = t.automated_water_duration_sec { self.timing.automated_water_duration_sec = w; }
            if let Some(m) = t.manual_water_duration_sec { self.timing.manual_water_duration_sec = m; }
            if let Some(p) = t.pump_speed_pct { self.timing.pump_speed_pct = p; }
        }
        if let Some(ref w) = patch.wifi {
            if let Some(m) = w.modem_sleep { self.wifi.modem_sleep = m; }
            if let Some(p) = w.tx_power { self.wifi.tx_power = p; }
            if let Some(f) = w.force_20mhz { self.wifi.force_20mhz = f; }
        }
        Ok(())
    }
}

/// Body accepted by `POST /api/config`. Unknown keys are ignored.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ConfigPatch {
    pub system: Option<SystemPatch>,
    pub gauges: Option<GaugesPatch>,
    pub safety: Option<SafetyPatch>,
    pub timing: Option<TimingPatch>,
    pub wifi: Option<WifiPatch>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct WifiPatch {
    pub modem_sleep: Option<bool>,
    pub tx_power: Option<i8>,
    pub force_20mhz: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TimingPatch {
    pub sensor_read_interval_sec: Option<u32>,
    pub automated_water_duration_sec: Option<u32>,
    pub manual_water_duration_sec: Option<u32>,
    pub pump_speed_pct: Option<u32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SystemPatch {
    pub hostname: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GaugesPatch {
    pub soil_moisture: Option<SoilMoisturePatch>,
    pub water_level: Option<WaterLevelPatch>,
    pub voltage: Option<VoltagePatch>,
    pub battery: Option<BatteryPatch>,
    pub temperature: Option<TemperaturePatch>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SoilMoisturePatch {
    pub trigger_threshold: Option<i32>,
    pub dry_adc: Option<i32>,
    pub wet_adc: Option<i32>,
    pub optimal_min: Option<i32>,
    pub optimal_max: Option<i32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct WaterLevelPatch {
    pub tank_height_cm: Option<f32>,
    pub sensor_offset_cm: Option<f32>,
    pub min_safe_level_cm: Option<f32>,
    pub refill_warning_pct: Option<f32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct VoltagePatch {
    pub min_warning: Option<f32>,
    pub max_warning: Option<f32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BatteryPatch {
    pub min_warning: Option<f32>,
    pub optimal_min: Option<f32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TemperaturePatch {
    pub optimal_max: Option<f32>,
    pub high_warning: Option<f32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SafetyPatch {
    pub max_continuous_pump_sec: Option<u32>,
    pub min_time_between_cycles_sec: Option<u32>,
    pub daily_watering_limit: Option<DailyLimitPatch>,
    pub night_watering_protection: Option<NightProtectionPatch>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DailyLimitPatch {
    pub enabled: Option<bool>,
    pub max_seconds_per_day: Option<u32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct NightProtectionPatch {
    pub enabled: Option<bool>,
    pub start_hour: Option<i32>,
    pub end_hour: Option<i32>,
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
        assert_eq!(c.gauges.soil_moisture.dry_adc, 1950);
        assert_eq!(c.gauges.soil_moisture.wet_adc, 670);
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
        assert_eq!(c.gauges.soil_moisture.dry_adc, 1950); // untouched
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
            r#"{"gauges":{"soil_moisture":{"trigger_threshold":30,"dry_adc":2100}},
                "safety":{"max_continuous_pump_sec":45,"min_time_between_cycles_sec":120},
                "ignored":true}"#,
        )
        .unwrap();
        c.apply_patch(&patch).unwrap();
        assert_eq!(c.gauges.soil_moisture.trigger_threshold, 30);
        assert_eq!(c.gauges.soil_moisture.dry_adc, 2100);
        assert_eq!(c.safety.max_continuous_pump_sec, 45);
        assert_eq!(c.safety.min_time_between_cycles_sec, 120);

        // Test hostname patch in system
        let sys_patch: ConfigPatch = serde_json::from_str(r#"{"system":{"hostname":"plants1"}}"#).unwrap();
        c.apply_patch(&sys_patch).unwrap();
        assert_eq!(c.system.hostname, "plants1");

        // Invalid hostname rejected
        let bad_host: ConfigPatch = serde_json::from_str(r#"{"system":{"hostname":"bad host!"}}"#).unwrap();
        assert!(c.apply_patch(&bad_host).is_err());
        assert_eq!(c.system.hostname, "plants1");
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

    #[test]
    fn pump_speed_patch_and_validation() {
        let mut c = AppConfig::default();
        assert_eq!(c.timing.pump_speed_pct, 60);

        let patch: ConfigPatch = serde_json::from_str(r#"{"timing":{"pump_speed_pct":75}}"#).unwrap();
        c.apply_patch(&patch).unwrap();
        assert_eq!(c.timing.pump_speed_pct, 75);

        // Under 10 rejected
        let too_low: ConfigPatch = serde_json::from_str(r#"{"timing":{"pump_speed_pct":5}}"#).unwrap();
        assert!(c.apply_patch(&too_low).is_err());
        assert_eq!(c.timing.pump_speed_pct, 75);

        // Over 100 rejected
        let too_high: ConfigPatch = serde_json::from_str(r#"{"timing":{"pump_speed_pct":105}}"#).unwrap();
        assert!(c.apply_patch(&too_high).is_err());
        assert_eq!(c.timing.pump_speed_pct, 75);
    }

    #[test]
    fn night_watering_protection_patch_and_validation() {
        let mut c = AppConfig::default();
        assert!(c.safety.night_watering_protection.enabled);
        assert_eq!(c.safety.night_watering_protection.start_hour, 22);
        assert_eq!(c.safety.night_watering_protection.end_hour, 9);

        let patch: ConfigPatch = serde_json::from_str(
            r#"{"safety":{"night_watering_protection":{"enabled":false,"start_hour":23,"end_hour":7}}}"#,
        )
        .unwrap();
        c.apply_patch(&patch).unwrap();
        assert!(!c.safety.night_watering_protection.enabled);
        assert_eq!(c.safety.night_watering_protection.start_hour, 23);
        assert_eq!(c.safety.night_watering_protection.end_hour, 7);

        // Invalid hour (> 23) rejected
        let bad_hour: ConfigPatch = serde_json::from_str(
            r#"{"safety":{"night_watering_protection":{"start_hour":24}}}"#,
        )
        .unwrap();
        assert!(c.apply_patch(&bad_hour).is_err());
        assert_eq!(c.safety.night_watering_protection.start_hour, 23);

        // Invalid hour (< 0) rejected
        let bad_neg: ConfigPatch = serde_json::from_str(
            r#"{"safety":{"night_watering_protection":{"end_hour":-1}}}"#,
        )
        .unwrap();
        assert!(c.apply_patch(&bad_neg).is_err());
        assert_eq!(c.safety.night_watering_protection.end_hour, 7);
    }

    #[test]
    fn wifi_settings_patch_and_validation() {
        let mut c = AppConfig::default();
        assert!(c.wifi.modem_sleep);
        assert_eq!(c.wifi.tx_power, 78);
        assert!(c.wifi.force_20mhz);
        assert!(c.wifi.radio_profile_v3);

        let patch: ConfigPatch = serde_json::from_str(
            r#"{"wifi":{"modem_sleep":false,"tx_power":60,"force_20mhz":false}}"#,
        )
        .unwrap();
        c.apply_patch(&patch).unwrap();
        assert!(!c.wifi.modem_sleep);
        assert_eq!(c.wifi.tx_power, 60);
        assert!(!c.wifi.force_20mhz);

        // Invalid power (< 8) rejected
        let bad_low: ConfigPatch = serde_json::from_str(r#"{"wifi":{"tx_power":5}}"#).unwrap();
        assert!(c.apply_patch(&bad_low).is_err());
        assert_eq!(c.wifi.tx_power, 60);

        // Invalid power (> 78) rejected
        let bad_high: ConfigPatch = serde_json::from_str(r#"{"wifi":{"tx_power":80}}"#).unwrap();
        assert!(c.apply_patch(&bad_high).is_err());
        assert_eq!(c.wifi.tx_power, 60);
    }
}
