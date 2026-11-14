//! JSON shapes served by the REST API and WebSocket (port of the handlers in
//! `web_server.cpp`).
//!
//! The field names and rounding are the contract with `data/app.js`; tests at
//! the bottom of this file pin them so a refactor cannot silently break the UI.

use crate::reading::{round1, round2, SensorReading};
use serde::{Deserialize, Serialize};

/// Entry of `GET /api/sensor-log`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LogEntry {
    pub timestamp: u32,
    pub soil_percent: f32,
    pub soil_raw: i32,
    pub water_level_cm: f32,
    pub water_percent: f32,
    pub distance_cm: f32,
    pub supply_voltage: f32,
    pub battery_voltage: f32,
    pub chip_temp: f32,
    pub pump_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub air_temp_c: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub humidity_pct: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light_lux: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ldr_percent: Option<f32>,
}

impl From<&SensorReading> for LogEntry {
    fn from(r: &SensorReading) -> Self {
        Self {
            timestamp: r.timestamp,
            soil_percent: round1(r.soil_percent),
            soil_raw: r.soil_raw_adc,
            water_level_cm: round1(r.water_level_cm),
            water_percent: round1(r.water_percent),
            distance_cm: round1(r.distance_cm),
            supply_voltage: round2(r.supply_voltage),
            battery_voltage: round2(r.battery_voltage),
            chip_temp: round1(r.chip_temp),
            pump_active: r.pump_active,
            air_temp_c: r.air_temp_c.map(round1),
            humidity_pct: r.humidity_pct.map(round1),
            light_lux: r.light_lux.map(round1),
            ldr_percent: r.ldr_percent.map(round1),
        }
    }
}

/// `sensors` object inside `GET /api/status` (no timestamp / pump flag).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct SensorsView {
    pub soil_percent: f32,
    pub soil_raw: i32,
    pub water_level_cm: f32,
    pub water_percent: f32,
    pub distance_cm: f32,
    pub supply_voltage: f32,
    pub battery_voltage: f32,
    pub chip_temp: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub air_temp_c: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub humidity_pct: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light_lux: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ldr_percent: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dli_yesterday: Option<f32>,
}

impl From<&SensorReading> for SensorsView {
    fn from(r: &SensorReading) -> Self {
        Self {
            soil_percent: round1(r.soil_percent),
            soil_raw: r.soil_raw_adc,
            water_level_cm: round1(r.water_level_cm),
            water_percent: round1(r.water_percent),
            distance_cm: round1(r.distance_cm),
            supply_voltage: round2(r.supply_voltage),
            battery_voltage: round2(r.battery_voltage),
            chip_temp: round1(r.chip_temp),
            air_temp_c: r.air_temp_c.map(round1),
            humidity_pct: r.humidity_pct.map(round1),
            light_lux: r.light_lux.map(round1),
            ldr_percent: r.ldr_percent.map(round1),
            dli_yesterday: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PumpView {
    pub active: bool,
    pub runtime_sec: u32,
    pub daily_used_sec: u32,
    pub cooldown_remaining_sec: u32,
    pub status_message: String,
    pub last_watered_ago_sec: Option<u32>,
    pub last_watered_duration_sec: u32,
    pub last_watered_text: String,
    pub quiet_hours_active: bool,
    pub quiet_hours_text: String,
}

/// `GET /api/status`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StatusResponse {
    pub hostname: String,
    pub ip: String,
    pub mdns: String,
    pub uptime_sec: u32,
    pub wifi_rssi: i32,
    pub time_unix: i64,
    pub ntp_synced: bool,
    pub sensors: SensorsView,
    pub pump: PumpView,
}

/// WebSocket push (`type == "telemetry"`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Telemetry {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub soil_percent: f32,
    pub soil_raw: i32,
    pub water_level_cm: f32,
    pub water_percent: f32,
    pub supply_voltage: f32,
    pub battery_voltage: f32,
    pub chip_temp: f32,
    pub pump_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub air_temp_c: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub humidity_pct: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light_lux: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ldr_percent: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dli_yesterday: Option<f32>,
}

impl From<&SensorReading> for Telemetry {
    fn from(r: &SensorReading) -> Self {
        Self {
            kind: "telemetry",
            soil_percent: round1(r.soil_percent),
            soil_raw: r.soil_raw_adc,
            water_level_cm: round1(r.water_level_cm),
            water_percent: round1(r.water_percent),
            supply_voltage: round2(r.supply_voltage),
            battery_voltage: round2(r.battery_voltage),
            chip_temp: round1(r.chip_temp),
            pump_active: r.pump_active,
            air_temp_c: r.air_temp_c.map(round1),
            humidity_pct: r.humidity_pct.map(round1),
            light_lux: r.light_lux.map(round1),
            ldr_percent: r.ldr_percent.map(round1),
            dli_yesterday: None,
        }
    }
}

/// `GET /api/sensor/health`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HealthResponse {
    pub preventive_resets: u32,
    pub adc_calibration: &'static str,
    pub wifi_rssi: i32,
    pub free_heap_bytes: u32,
    pub chip_temp_c: f32,
}

/// `POST /api/water` response.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WaterResponse {
    pub success: bool,
    pub message: String,
    pub duration_sec: u32,
}

/// `POST /api/sensor/ultrasonic/reset` response.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ResetResponse {
    pub success: bool,
    pub preventive_resets: u32,
}

/// Serialises any API value. Plain data structs cannot fail to serialise.
pub fn to_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("API structs serialise infallibly")
}

/// `GET /api/sensor-log?n=` parsing: default 10, clamp to 1..=150; missing,
/// non-numeric or non-positive values fall back to 10.
pub fn parse_log_count(n: Option<&str>) -> usize {
    match n.and_then(|s| s.trim().parse::<i64>().ok()) {
        Some(v) if v > 0 => (v as usize).min(crate::reading::MAX_HISTORY_CAPACITY),
        _ => 10,
    }
}

/// Extracts the `n` query parameter from a request URI such as `/api/sensor-log?n=15`.
pub fn query_param<'a>(uri: &'a str, key: &str) -> Option<&'a str> {
    let query = uri.split_once('?')?.1;
    query.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        (k == key).then_some(v)
    })
}

#[derive(Deserialize)]
struct WaterRequest {
    duration: Option<u32>,
}

/// `POST /api/water` body: `{"duration": <seconds>}`. An empty/invalid body or
/// a missing/invalid `duration` falls back to `default_sec`
/// (`timing.manual_water_duration_sec`).
pub fn parse_water_duration(body: &[u8], default_sec: u32) -> u32 {
    if body.is_empty() {
        return default_sec;
    }
    serde_json::from_slice::<WaterRequest>(body).ok().and_then(|r| r.duration).unwrap_or(default_sec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn sample() -> SensorReading {
        SensorReading {
            timestamp: 1234,
            soil_percent: 41.46,
            soil_raw_adc: 1987,
            water_level_cm: 17.84,
            water_percent: 66.07,
            distance_cm: 12.16,
            supply_voltage: 4.987,
            battery_voltage: 3.7149,
            chip_temp: 38.26,
            pump_active: true,
            air_temp_c: Some(24.52),
            humidity_pct: Some(58.34),
            light_lux: Some(450.6),
            ldr_percent: Some(65.2),
        }
    }

    fn json_of<T: Serialize>(v: &T) -> Value {
        serde_json::from_str(&to_json(v)).unwrap()
    }

    #[test]
    fn log_entry_has_every_field_the_ui_reads_with_cpp_rounding() {
        let v = json_of(&LogEntry::from(&sample()));
        // app.js renderLogTable() reads these:
        assert_eq!(v["timestamp"], 1234);
        assert_eq!(v["soil_percent"], 41.5);
        assert_eq!(v["water_level_cm"], 17.8);
        assert_eq!(v["supply_voltage"], 4.99);
        assert_eq!(v["battery_voltage"], 3.71);
        assert_eq!(v["chip_temp"], 38.3);
        // and the rest of the original schema
        assert_eq!(v["soil_raw"], 1987);
        assert_eq!(v["water_percent"], 66.1);
        assert_eq!(v["distance_cm"], 12.2);
        assert_eq!(v["pump_active"], true);
        assert_eq!(v["air_temp_c"], 24.5);
        assert_eq!(v["humidity_pct"], 58.3);
        assert_eq!(v["light_lux"], 450.6);
        assert_eq!(v["ldr_percent"], 65.2);
        assert_eq!(v.as_object().unwrap().len(), 14);
    }

    #[test]
    fn telemetry_matches_websocket_contract() {
        let v = json_of(&Telemetry::from(&sample()));
        assert_eq!(v["type"], "telemetry"); // app.js: if (data.type === 'telemetry')
        for key in [
            "soil_percent",
            "soil_raw",
            "water_level_cm",
            "water_percent",
            "supply_voltage",
            "battery_voltage",
            "chip_temp",
            "pump_active",
            "air_temp_c",
            "humidity_pct",
            "light_lux",
            "ldr_percent",
        ] {
            assert!(v.get(key).is_some(), "telemetry missing {key}");
        }
        assert!(v.get("distance_cm").is_none(), "C++ telemetry omitted distance_cm");
        assert_eq!(v.as_object().unwrap().len(), 13);
    }

    #[test]
    fn status_matches_ui_contract() {
        let r = sample();
        let status = StatusResponse {
            hostname: "plants".into(),
            ip: "192.168.1.50".into(),
            mdns: "plants.local".into(),
            uptime_sec: 3661,
            wifi_rssi: -61,
            time_unix: 1794188583,
            ntp_synced: true,
            sensors: SensorsView::from(&r),
            pump: PumpView {
                active: false,
                runtime_sec: 0,
                daily_used_sec: 20,
                cooldown_remaining_sec: 120,
                status_message: "Stopped: Target duration or safety runtime reached".into(),
                last_watered_ago_sec: Some(180),
                last_watered_duration_sec: 15,
                last_watered_text: "3m ago (15s)".into(),
                quiet_hours_active: false,
                quiet_hours_text: "Inactive (22:00 - 09:00)".into(),
            },
        };
        let v = json_of(&status);
        // app.js updateFullStatusUI():
        assert_eq!(v["hostname"], "plants");
        assert_eq!(v["ip"], "192.168.1.50");
        assert_eq!(v["wifi_rssi"], -61);
        assert_eq!(v["uptime_sec"], 3661);
        assert_eq!(v["time_unix"], 1794188583);
        assert_eq!(v["ntp_synced"], true);
        assert_eq!(v["pump"]["status_message"], "Stopped: Target duration or safety runtime reached");
        assert_eq!(v["pump"]["cooldown_remaining_sec"], 120);
        assert_eq!(v["pump"]["daily_used_sec"], 20);
        assert_eq!(v["pump"]["active"], false);
        assert_eq!(v["pump"]["quiet_hours_active"], false);
        assert_eq!(v["pump"]["quiet_hours_text"], "Inactive (22:00 - 09:00)");
        assert_eq!(v["sensors"]["soil_percent"], 41.5);
        assert!(v["sensors"].get("pump_active").is_none());
        assert!(v["mdns"].is_string());
    }

    #[test]
    fn health_and_action_responses_have_expected_keys() {
        let h = json_of(&HealthResponse {
            preventive_resets: 2,
            adc_calibration: "eFuse factory curve active",
            wifi_rssi: -70,
            free_heap_bytes: 123456,
            chip_temp_c: 35.5,
        });
        for k in ["preventive_resets", "adc_calibration", "wifi_rssi", "free_heap_bytes", "chip_temp_c"] {
            assert!(h.get(k).is_some(), "health missing {k}");
        }
        let w = json_of(&WaterResponse { success: false, message: "nope".into(), duration_sec: 10 });
        assert_eq!(w["success"], false);
        assert_eq!(w["message"], "nope");
        assert_eq!(w["duration_sec"], 10);
        let r = json_of(&ResetResponse { success: true, preventive_resets: 3 });
        assert_eq!(r["preventive_resets"], 3); // app.js alert reads result.preventive_resets
    }

    #[test]
    fn sensor_fault_distance_serialises_as_minus_one() {
        let mut r = sample();
        r.distance_cm = -1.0;
        assert_eq!(json_of(&LogEntry::from(&r))["distance_cm"], -1.0);
    }

    #[test]
    fn log_count_parsing() {
        assert_eq!(parse_log_count(None), 10);
        assert_eq!(parse_log_count(Some("15")), 15);
        assert_eq!(parse_log_count(Some("0")), 10);
        assert_eq!(parse_log_count(Some("abc")), 10);
        assert_eq!(parse_log_count(Some("")), 10);
        assert_eq!(parse_log_count(Some("-4")), 10);
        assert_eq!(parse_log_count(Some("100")), 100);
        assert_eq!(parse_log_count(Some("150")), 150);
        assert_eq!(parse_log_count(Some("5000")), 150);
    }

    #[test]
    fn query_string_extraction() {
        assert_eq!(query_param("/api/sensor-log?n=15", "n"), Some("15"));
        assert_eq!(query_param("/api/sensor-log?x=1&n=7", "n"), Some("7"));
        assert_eq!(query_param("/api/sensor-log", "n"), None);
        assert_eq!(query_param("/api/sensor-log?n", "n"), Some(""));
        assert_eq!(query_param("/api/sensor-log?m=1", "n"), None);
    }

    #[test]
    fn water_duration_parsing() {
        assert_eq!(parse_water_duration(br#"{"duration":25}"#, 10), 25);
        assert_eq!(parse_water_duration(b"", 10), 10);
        assert_eq!(parse_water_duration(b"not json", 10), 10);
        assert_eq!(parse_water_duration(br#"{"other":1}"#, 10), 10);
        assert_eq!(parse_water_duration(br#"{"duration":-5}"#, 10), 10);
        assert_eq!(parse_water_duration(br#"{"duration":"x"}"#, 10), 10);
    }
}
