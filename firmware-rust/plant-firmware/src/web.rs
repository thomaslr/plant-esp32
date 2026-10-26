//! Web Server and REST API Implementation.
//!
//! Serves the embedded web UI and provides REST endpoints matching the C++ firmware contract.

use std::sync::{Arc, Mutex};

use embedded_svc::http::server::Request;
use embedded_svc::http::{Headers, Method};
use embedded_svc::io::{Read, Write};
use esp_idf_svc::http::server::{Configuration as ServerConfig, EspHttpConnection, EspHttpServer};
use esp_idf_svc::io::EspIOError;
use log::info;
use plant_core::api::{
    parse_log_count, parse_water_duration, query_param, to_json, HealthResponse, LogEntry,
    PumpView, ResetResponse, SensorsView, StatusResponse, WaterResponse,
};
use plant_core::config::{AppConfig, ConfigPatch};
use plant_core::reading::SensorReading;
use plant_core::ring_buffer::RingBuffer;
use plant_core::safety::SafetyManager;

use crate::pump::HardwarePumpGate;
use crate::sensors::HardwareSensorManager;
use crate::storage::NvsConfigStorage;

// Embed web UI assets directly into firmware flash binary
static INDEX_HTML: &str = include_str!("../../../data/index.html");
static APP_JS: &str = include_str!("../../../data/app.js");
static STYLE_CSS: &str = include_str!("../../../data/style.css");

pub struct AppState {
    pub config: AppConfig,
    pub safety: SafetyManager<HardwarePumpGate>,
    pub sensors: HardwareSensorManager,
    pub history: RingBuffer<SensorReading>,
    pub storage: NvsConfigStorage,
    pub ip_address: String,
    pub wifi_rssi: i32,
}

pub fn create_web_server(
    state: Arc<Mutex<AppState>>,
) -> Result<EspHttpServer<'static>, EspIOError> {
    let server_config = ServerConfig {
        stack_size: 10240,
        ..Default::default()
    };
    let mut server = EspHttpServer::new(&server_config)?;

    // 1. Static Asset: GET / and GET /index.html
    server.fn_handler("/", Method::Get, |req| {
        send_file(req, "text/html; charset=utf-8", INDEX_HTML)
    })?;
    server.fn_handler("/index.html", Method::Get, |req| {
        send_file(req, "text/html; charset=utf-8", INDEX_HTML)
    })?;

    // 2. Static Asset: GET /app.js
    server.fn_handler("/app.js", Method::Get, |req| {
        send_file(req, "application/javascript", APP_JS)
    })?;

    // 3. Static Asset: GET /style.css
    server.fn_handler("/style.css", Method::Get, |req| {
        send_file(req, "text/css", STYLE_CSS)
    })?;

    // 4. GET /api/status
    let s_status = Arc::clone(&state);
    server.fn_handler("/api/status", Method::Get, move |req| {
        let (json, status_code) = {
            let app = s_status.lock().unwrap();
            let latest = app.history.latest();
            let now_ms = unsafe { (esp_idf_svc::sys::esp_timer_get_time() / 1000) as u32 };
            let uptime_sec = now_ms / 1000;
            let mdns = format!("{}.local", app.config.system.hostname);

            let sensors = match latest {
                Some(ref r) => SensorsView::from(r),
                None => SensorsView {
                    soil_percent: 0.0,
                    soil_raw: 0,
                    water_level_cm: 0.0,
                    water_percent: 0.0,
                    distance_cm: 0.0,
                    supply_voltage: 0.0,
                    battery_voltage: 0.0,
                    chip_temp: 0.0,
                },
            };

            let pump = PumpView {
                active: app.safety.is_pump_active(),
                runtime_sec: app.safety.active_runtime_sec(now_ms),
                daily_used_sec: app.safety.daily_runtime_sec(),
                cooldown_remaining_sec: app.safety.seconds_until_next_allowed(now_ms),
                status_message: app.safety.last_inhibit_reason().to_string(),
            };

            let status = StatusResponse {
                hostname: app.config.system.hostname.clone(),
                ip: app.ip_address.clone(),
                mdns,
                uptime_sec,
                wifi_rssi: app.wifi_rssi,
                sensors,
                pump,
            };

            (to_json(&status), 200)
        };
        send_json(req, status_code, &json)
    })?;

    // 5. GET /api/sensor-log
    let s_log = Arc::clone(&state);
    server.fn_handler("/api/sensor-log", Method::Get, move |req| {
        let uri = req.uri();
        let q = query_param(uri, "n");
        let count = parse_log_count(q);

        let json = {
            let app = s_log.lock().unwrap();
            let recent = app.history.recent(count);
            let entries: Vec<LogEntry> = recent.iter().map(LogEntry::from).collect();
            to_json(&entries)
        };
        send_json(req, 200, &json)
    })?;

    // 6. GET /api/config
    let s_cfg_get = Arc::clone(&state);
    server.fn_handler("/api/config", Method::Get, move |req| {
        let json = {
            let app = s_cfg_get.lock().unwrap();
            to_json(&app.config)
        };
        send_json(req, 200, &json)
    })?;

    // 7. POST /api/config
    let s_cfg_post = Arc::clone(&state);
    server.fn_handler("/api/config", Method::Post, move |mut req| {
        let len = req.content_len().unwrap_or(0) as usize;
        let mut buf = vec![0u8; len];
        let _ = req.read_exact(&mut buf);

        let res = {
            let mut app = s_cfg_post.lock().unwrap();
            match serde_json::from_slice::<ConfigPatch>(&buf) {
                Ok(patch) => match app.config.apply_patch(&patch) {
                    Ok(()) => {
                        let _ = app.storage.save(&app.config);
                        let cfg = app.config.clone();
                        app.safety.update_config(&cfg);
                        (200, r#"{"status":"success"}"#)
                    }
                    Err(_) => (400, r#"{"error":"Validation failed"}"#),
                },
                Err(_) => (400, r#"{"error":"Invalid JSON"}"#),
            }
        };
        send_json(req, res.0, res.1)
    })?;

    // 8. POST /api/water
    let s_water = Arc::clone(&state);
    server.fn_handler("/api/water", Method::Post, move |mut req| {
        let len = req.content_len().unwrap_or(0) as usize;
        let mut buf = vec![0u8; len];
        let _ = req.read_exact(&mut buf);

        let (status_code, json) = {
            let mut app = s_water.lock().unwrap();
            let default_duration = app.config.timing.manual_water_duration_sec;
            let duration = parse_water_duration(&buf, default_duration);

            let distance = app.sensors.read_ultrasonic_distance_cm();
            let water_depth = if distance > 0.0 {
                app.config.gauges.water_level.tank_height_cm - distance
            } else {
                0.0
            };

            let now_ms = unsafe { (esp_idf_svc::sys::esp_timer_get_time() / 1000) as u32 };
            let result = app.safety.request_watering(
                now_ms,
                duration,
                "Manual Web Request",
                water_depth,
                None, // Skip night protection for manual request
            );

            match result {
                Ok(granted_sec) => {
                    let resp = WaterResponse {
                        success: true,
                        message: "Watering started".to_string(),
                        duration_sec: granted_sec,
                    };
                    (200, to_json(&resp))
                }
                Err(inhibit) => {
                    let resp = WaterResponse {
                        success: false,
                        message: inhibit.to_string(),
                        duration_sec: duration,
                    };
                    (409, to_json(&resp))
                }
            }
        };
        send_json(req, status_code, &json)
    })?;

    // 9. GET /api/sensor/health
    let s_health = Arc::clone(&state);
    server.fn_handler("/api/sensor/health", Method::Get, move |req| {
        let (json, status_code) = {
            let app = s_health.lock().unwrap();
            let resets = app.sensors.preventive_resets_count();
            let heap = unsafe { esp_idf_svc::sys::esp_get_free_heap_size() };
            let temp = app.sensors.read_chip_temperature();
            let health = HealthResponse {
                preventive_resets: resets,
                adc_calibration: "eFuse factory curve active",
                wifi_rssi: app.wifi_rssi,
                free_heap_bytes: heap,
                chip_temp_c: temp,
            };
            (to_json(&health), 200)
        };
        send_json(req, status_code, &json)
    })?;

    // 10. POST /api/sensor/ultrasonic/reset
    let s_reset = Arc::clone(&state);
    server.fn_handler("/api/sensor/ultrasonic/reset", Method::Post, move |req| {
        let json = {
            let app = s_reset.lock().unwrap();
            app.sensors.force_ultrasonic_reset();
            let count = app.sensors.preventive_resets_count();
            let reset = ResetResponse {
                success: true,
                preventive_resets: count,
            };
            to_json(&reset)
        };
        send_json(req, 200, &json)
    })?;

    info!("[HTTP] Web server registered all endpoints successfully on port 80");
    Ok(server)
}

fn send_file(
    req: Request<&mut EspHttpConnection>,
    content_type: &str,
    content: &str,
) -> Result<(), EspIOError> {
    let headers = [
        ("Content-Type", content_type),
        ("Cache-Control", "public, max-age=3600"),
    ];
    let mut resp = req.into_response(200, Some("OK"), &headers)?;
    let _ = resp.write_all(content.as_bytes());
    Ok(())
}

fn send_json(
    req: Request<&mut EspHttpConnection>,
    status: u16,
    json: &str,
) -> Result<(), EspIOError> {
    let headers = [
        ("Content-Type", "application/json"),
        ("Access-Control-Allow-Origin", "*"),
        ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
        ("Access-Control-Allow-Headers", "*"),
    ];
    let mut resp = req.into_response(status, None, &headers)?;
    let _ = resp.write_all(json.as_bytes());
    Ok(())
}
