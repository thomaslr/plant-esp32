//! Web Server and REST API Implementation.
//!
//! Serves the embedded web UI and provides REST endpoints matching the C++ firmware contract.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use embedded_svc::http::server::Request;
use embedded_svc::http::{Headers, Method};
use embedded_svc::io::{Read, Write};
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::http::server::{
    Configuration as ServerConfig, EspHttpConnection, EspHttpServer, KeepAlive,
};
use esp_idf_svc::io::EspIOError;
use log::{error, info};
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

// Embed web UI assets directly into firmware flash binary (both raw and pre-compressed Gzip)
static INDEX_HTML: &str = include_str!("../../../data/index.html");
static INDEX_HTML_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/index.html.gz"));
static APP_JS: &str = include_str!("../../../data/app.js");
static APP_JS_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/app.js.gz"));
static STYLE_CSS: &str = include_str!("../../../data/style.css");
static STYLE_CSS_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/style.css.gz"));
static SETUP_HTML: &str = include_str!("../../../data/setup.html");
static SETUP_HTML_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/setup.html.gz"));
static FAVICON_SVG: &str = include_str!("../../../data/favicon.svg");
static FAVICON_SVG_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/favicon.svg.gz"));
static MANIFEST_JSON: &str = include_str!("../../../data/manifest.json");
static MANIFEST_JSON_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/manifest.json.gz"));
static SW_JS: &str = include_str!("../../../data/sw.js");
static SW_JS_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/sw.js.gz"));
static ICON_192_PNG: &[u8] = include_bytes!("../../../data/icon-192.png");
static ICON_192_PNG_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/icon-192.png.gz"));
static ICON_512_PNG: &[u8] = include_bytes!("../../../data/icon-512.png");
static ICON_512_PNG_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/icon-512.png.gz"));

pub struct AppState {
    pub config: AppConfig,
    pub safety: SafetyManager<HardwarePumpGate>,
    pub sensors: Arc<HardwareSensorManager>,
    pub history: RingBuffer<SensorReading>,
    pub storage: NvsConfigStorage,
    pub ip_address: String,
    pub wifi_rssi: i32,
    pub pump_speed: Arc<AtomicU32>,
}

pub fn create_web_server(
    state: Arc<Mutex<AppState>>,
) -> Result<EspHttpServer<'static>, EspIOError> {
    let server_config = ServerConfig {
        stack_size: 10240,
        max_open_sockets: 10,
        lru_purge_enable: true,
        keep_alive: Some(KeepAlive {
            idle_secs: 5,
            interval_secs: 5,
            probe_count: 3,
        }),
        ..Default::default()
    };
    let mut server = EspHttpServer::new(&server_config)?;

    // 1. Static Asset: GET / and GET /index.html
    server.fn_handler("/", Method::Get, |req| {
        send_file(req, "text/html; charset=utf-8", INDEX_HTML, INDEX_HTML_GZ)
    })?;
    server.fn_handler("/index.html", Method::Get, |req| {
        send_file(req, "text/html; charset=utf-8", INDEX_HTML, INDEX_HTML_GZ)
    })?;

    // 2. Static Asset: GET /app.js
    server.fn_handler("/app.js", Method::Get, |req| {
        send_file(req, "application/javascript", APP_JS, APP_JS_GZ)
    })?;

    // 3. Static Asset: GET /style.css
    server.fn_handler("/style.css", Method::Get, |req| {
        send_file(req, "text/css", STYLE_CSS, STYLE_CSS_GZ)
    })?;

    // 4. Static Asset: GET /favicon.svg and GET /favicon.ico
    server.fn_handler("/favicon.svg", Method::Get, |req| {
        send_file(req, "image/svg+xml", FAVICON_SVG, FAVICON_SVG_GZ)
    })?;
    server.fn_handler("/favicon.ico", Method::Get, |req| {
        send_file(req, "image/svg+xml", FAVICON_SVG, FAVICON_SVG_GZ)
    })?;

    // 4B. PWA Manifest, Service Worker, and App Icons
    server.fn_handler("/manifest.json", Method::Get, |req| {
        send_file(req, "application/manifest+json", MANIFEST_JSON, MANIFEST_JSON_GZ)
    })?;
    server.fn_handler("/sw.js", Method::Get, |req| {
        send_file(req, "application/javascript", SW_JS, SW_JS_GZ)
    })?;
    server.fn_handler("/icon-192.png", Method::Get, |req| {
        send_binary(req, "image/png", ICON_192_PNG, ICON_192_PNG_GZ)
    })?;
    server.fn_handler("/icon-512.png", Method::Get, |req| {
        send_binary(req, "image/png", ICON_512_PNG, ICON_512_PNG_GZ)
    })?;

    // 5. GET /api/status
    let s_status = Arc::clone(&state);
    server.fn_handler("/api/status", Method::Get, move |req| {
        let (json, status_code) = {
            let app = s_status.lock().unwrap();
            let latest = app.history.latest();
            let now_ms = unsafe { (esp_idf_svc::sys::esp_timer_get_time() / 1000) as u32 };
            let uptime_sec = now_ms / 1000;
            let mdns = format!("{}.local", app.config.system.hostname);

            let unix_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            let mut sensors = match latest {
                Some(ref r) => SensorsView::from(r),
                None => SensorsView::default(),
            };

            // Calculate yesterday's Daily Light Integral (DLI) from historical telemetry
            if let Some(day) = plant_core::timeutil::local_day(unix_secs, app.config.system.gmt_offset) {
                let all_readings = app.history.all();
                sensors.dli_yesterday = plant_core::dli::calculate_daily_dli(&all_readings, day - 1, app.config.system.gmt_offset);
            }

            let last_watered_ago_sec = app.safety.last_watered_ago_sec(now_ms);
            let last_watered_duration_sec = app.safety.last_watered_duration_sec();
            let last_watered_text = plant_core::timeutil::format_last_watered(
                last_watered_ago_sec,
                last_watered_duration_sec,
            );
            let current_hour = plant_core::timeutil::local_hour(unix_secs, app.config.system.gmt_offset);
            let (quiet_hours_active, quiet_hours_text) = plant_core::timeutil::format_night_protection(
                app.config.safety.night_watering_protection.enabled,
                app.config.safety.night_watering_protection.start_hour,
                app.config.safety.night_watering_protection.end_hour,
                current_hour,
            );

            let pump = PumpView {
                active: app.safety.is_pump_active(),
                runtime_sec: app.safety.active_runtime_sec(now_ms),
                daily_used_sec: app.safety.daily_runtime_sec(),
                cooldown_remaining_sec: app.safety.seconds_until_next_allowed(now_ms),
                status_message: app.safety.last_inhibit_reason().to_string(),
                last_watered_ago_sec,
                last_watered_duration_sec,
                last_watered_text,
                quiet_hours_active,
                quiet_hours_text,
            };

            let status = StatusResponse {
                hostname: app.config.system.hostname.clone(),
                ip: app.ip_address.clone(),
                mdns,
                uptime_sec,
                wifi_rssi: app.wifi_rssi,
                time_unix: unix_secs,
                ntp_synced: unix_secs > 1_600_000_000,
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
                        app.pump_speed.store(cfg.timing.pump_speed_pct, Ordering::Relaxed);
                        info!("[CONFIG] Updated pump PWM speed to {}%", cfg.timing.pump_speed_pct);
                        crate::wifi::apply_wifi_radio_config(&cfg.wifi);

                        // Recalculate latest cached reading in history with new calibration thresholds
                        if let Some(mut latest) = app.history.latest() {
                            latest.soil_percent = plant_core::sensors::soil_percent(
                                latest.soil_raw_adc,
                                app.config.gauges.soil_moisture.dry_adc,
                                app.config.gauges.soil_moisture.wet_adc,
                            );
                            app.history.push(latest);
                        }

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

        let (default_duration, tank_height_cm, sensors) = {
            let app = s_water.lock().unwrap();
            (
                app.config.timing.manual_water_duration_sec,
                app.config.gauges.water_level.tank_height_cm,
                Arc::clone(&app.sensors),
            )
        };
        let duration = parse_water_duration(&buf, default_duration);
        let distance = sensors.read_ultrasonic_distance_cm();
        let water_depth = if distance > 0.0 {
            tank_height_cm - distance
        } else {
            0.0
        };

        let (status_code, json) = {
            let mut app = s_water.lock().unwrap();
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
        let (resets, wifi_rssi, sensors) = {
            let app = s_health.lock().unwrap();
            (
                app.sensors.preventive_resets_count(),
                app.wifi_rssi,
                Arc::clone(&app.sensors),
            )
        };
        let heap = unsafe { esp_idf_svc::sys::esp_get_free_heap_size() };
        let temp = sensors.read_chip_temperature();
        let health = HealthResponse {
            preventive_resets: resets,
            adc_calibration: "eFuse factory curve active",
            wifi_rssi,
            free_heap_bytes: heap,
            chip_temp_c: temp,
        };
        send_json(req, 200, &to_json(&health))
    })?;

    // 10. POST /api/sensor/ultrasonic/reset
    let s_reset = Arc::clone(&state);
    server.fn_handler("/api/sensor/ultrasonic/reset", Method::Post, move |req| {
        let sensors = {
            let app = s_reset.lock().unwrap();
            Arc::clone(&app.sensors)
        };
        sensors.force_ultrasonic_reset();
        let count = sensors.preventive_resets_count();
        let reset = ResetResponse {
            success: true,
            preventive_resets: count,
        };
        send_json(req, 200, &to_json(&reset))
    })?;

    // 10b. POST & GET /api/sensor/soil/sample (Live high-precision 15-reading calibration sample)
    let s_soil_cal_post = Arc::clone(&state);
    server.fn_handler("/api/sensor/soil/sample", Method::Post, move |mut req| {
        let len = req.content_len().unwrap_or(0) as usize;
        let mut buf = vec![0u8; len];
        let _ = req.read_exact(&mut buf);

        let target = if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&buf) {
            v.get("target")
                .or_else(|| v.get("type"))
                .and_then(|val| val.as_str())
                .map(|s| s.to_ascii_lowercase())
        } else {
            None
        };

        let sensors = {
            let app = s_soil_cal_post.lock().unwrap();
            Arc::clone(&app.sensors)
        };
        let raw = sensors.sample_soil_moisture_calibration();

        let (raw_mv, soil_pct, saved) = {
            let mut app = s_soil_cal_post.lock().unwrap();

            let mut was_saved = false;
            if let Some(ref t) = target {
                if t == "dry" {
                    app.config.gauges.soil_moisture.dry_adc = raw;
                    let _ = app.storage.save(&app.config);
                    let _ = app.storage.save_soil_calibration(Some(raw), None);
                    was_saved = true;
                    info!("[CALIBRATION] Auto-saved dry_adc = {} mV to NVS flash", raw);
                } else if t == "wet" {
                    app.config.gauges.soil_moisture.wet_adc = raw;
                    let _ = app.storage.save(&app.config);
                    let _ = app.storage.save_soil_calibration(None, Some(raw));
                    was_saved = true;
                    info!("[CALIBRATION] Auto-saved wet_adc = {} mV to NVS flash", raw);
                }

                // Immediately update latest reading in history so /api/status matches the calibration instantly
                if let Some(mut latest) = app.history.latest() {
                    latest.soil_raw_adc = raw;
                    latest.soil_percent = plant_core::sensors::soil_percent(
                        raw,
                        app.config.gauges.soil_moisture.dry_adc,
                        app.config.gauges.soil_moisture.wet_adc,
                    );
                    app.history.push(latest);
                }
            }

            let dry = app.config.gauges.soil_moisture.dry_adc;
            let wet = app.config.gauges.soil_moisture.wet_adc;
            let pct = plant_core::sensors::soil_percent(raw, dry, wet);
            (raw, pct, was_saved)
        };
        let body = format!(
            r#"{{"success":true,"soil_raw":{},"soil_percent":{:.1},"saved":{}}}"#,
            raw_mv, soil_pct, saved
        );
        send_json(req, 200, &body)
    })?;

    let s_soil_cal_get = Arc::clone(&state);
    server.fn_handler("/api/sensor/soil/sample", Method::Get, move |req| {
        let sensors = {
            let app = s_soil_cal_get.lock().unwrap();
            Arc::clone(&app.sensors)
        };
        let raw = sensors.sample_soil_moisture_calibration();
        let (raw_mv, soil_pct) = {
            let app = s_soil_cal_get.lock().unwrap();
            let dry = app.config.gauges.soil_moisture.dry_adc;
            let wet = app.config.gauges.soil_moisture.wet_adc;
            let pct = plant_core::sensors::soil_percent(raw, dry, wet);
            (raw, pct)
        };
        let body = format!(
            r#"{{"success":true,"soil_raw":{},"soil_percent":{:.1}}}"#,
            raw_mv, soil_pct
        );
        send_json(req, 200, &body)
    })?;

    // 11. POST /api/wifi/reset
    let s_wifi_reset = Arc::clone(&state);
    server.fn_handler("/api/wifi/reset", Method::Post, move |req| {
        info!("[HTTP] Wi-Fi reset requested via REST API. Clearing credentials and rebooting...");
        {
            let app = s_wifi_reset.lock().unwrap();
            let _ = app.storage.clear_wifi();
        }
        let res = send_json(
            req,
            200,
            r#"{"status":"success","message":"Wi-Fi reset. Restarting into setup mode..."}"#,
        );
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(1500));
            unsafe {
                esp_idf_svc::sys::esp_restart();
            }
        });
        res
    })?;

    // 12. POST /api/system/reboot
    server.fn_handler("/api/system/reboot", Method::Post, move |req| {
        info!("[HTTP] System reboot requested via REST API. Restarting...");
        let res = send_json(
            req,
            200,
            r#"{"status":"success","message":"Device is rebooting..."}"#,
        );
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(1500));
            unsafe {
                esp_idf_svc::sys::esp_restart();
            }
        });
        res
    })?;

    // 13. GET /api/ota/status
    server.fn_handler("/api/ota/status", Method::Get, move |req| {
        let status = crate::ota::get_ota_status();
        let json = to_json(&status);
        send_json(req, 200, &json)
    })?;

    // 13. POST /api/ota/update
    server.fn_handler("/api/ota/update", Method::Post, move |mut req| {
        let len = req.content_len().unwrap_or(0) as usize;
        let mut buf = vec![0u8; len];
        let _ = req.read_exact(&mut buf);

        let url = if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&buf) {
            v.get("url")
                .and_then(|u| u.as_str())
                .unwrap_or(crate::ota::DEFAULT_GITHUB_OTA_URL)
                .to_string()
        } else {
            crate::ota::DEFAULT_GITHUB_OTA_URL.to_string()
        };

        info!("[HTTP] Remote OTA update requested from: {}", url);
        // The HTTPS download needs a TLS handshake plus flash writes, which overflows
        // the ~3 KB default std thread stack on ESP-IDF (Stack protection fault).
        // Give the OTA worker a dedicated, generous stack.
        let spawn_result = std::thread::Builder::new()
            .name("OtaDownload".to_string())
            .stack_size(32 * 1024)
            .spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(500));
                if let Err(e) = crate::ota::update_from_url(&url) {
                    error!("[OTA] Remote OTA update failed: {}", e);
                }
            });
        if let Err(e) = spawn_result {
            error!("[OTA] Failed to spawn OTA worker thread: {:?}", e);
        }

        send_json(
            req,
            200,
            r#"{"status":"started","message":"OTA update initiated in background. Device will reboot when complete."}"#,
        )
    })?;

    // 14. POST /api/ota/upload
    server.fn_handler("/api/ota/upload", Method::Post, move |mut req| {
        let len = req.content_len().unwrap_or(0) as usize;
        info!("[HTTP] Direct OTA upload received ({} bytes)", len);
        if len < 500_000 {
            return send_json(req, 400, r#"{"error":"File too small to be valid firmware"}"#);
        }

        match crate::ota::flash_stream(&mut req, len) {
            Ok(()) => send_json(
                req,
                200,
                r#"{"status":"success","message":"Firmware flashed! Rebooting..."}"#,
            ),
            Err(e) => {
                error!("[OTA] Flash stream error: {}", e);
                send_json(req, 500, &format!(r#"{{"error":"{}"}}"#, e))
            }
        }
    })?;

    info!("[HTTP] Web server registered all endpoints successfully on port 80");
    Ok(server)
}

/// Creates a dedicated lightweight HTTP server for the SoftAP Captive Portal provisioning mode.
pub fn create_provisioning_server(
    storage: Arc<Mutex<NvsConfigStorage>>,
    scanned_ssids: Arc<Mutex<Vec<String>>>,
) -> Result<EspHttpServer<'static>, EspIOError> {
    let server_config = ServerConfig {
        stack_size: 10240,
        max_open_sockets: 8,
        lru_purge_enable: true,
        ..Default::default()
    };
    let mut server = EspHttpServer::new(&server_config)?;

    // 1. Setup Page (GET / and GET /setup)
    server.fn_handler("/", Method::Get, |req| {
        send_file(req, "text/html; charset=utf-8", SETUP_HTML, SETUP_HTML_GZ)
    })?;
    server.fn_handler("/setup", Method::Get, |req| {
        send_file(req, "text/html; charset=utf-8", SETUP_HTML, SETUP_HTML_GZ)
    })?;

    // 2. Captive Portal Detection endpoints (iOS, Android, Windows)
    server.fn_handler("/hotspot-detect.html", Method::Get, |req| {
        send_file(req, "text/html; charset=utf-8", SETUP_HTML, SETUP_HTML_GZ)
    })?;
    server.fn_handler("/generate_204", Method::Get, |req| {
        send_file(req, "text/html; charset=utf-8", SETUP_HTML, SETUP_HTML_GZ)
    })?;
    server.fn_handler("/gen_204", Method::Get, |req| {
        send_file(req, "text/html; charset=utf-8", SETUP_HTML, SETUP_HTML_GZ)
    })?;
    server.fn_handler("/ncsi.txt", Method::Get, |req| {
        send_file(req, "text/html; charset=utf-8", SETUP_HTML, SETUP_HTML_GZ)
    })?;
    server.fn_handler("/connecttest.txt", Method::Get, |req| {
        send_file(req, "text/html; charset=utf-8", SETUP_HTML, SETUP_HTML_GZ)
    })?;

    // Favicon endpoints
    server.fn_handler("/favicon.svg", Method::Get, |req| {
        send_file(req, "image/svg+xml", FAVICON_SVG, FAVICON_SVG_GZ)
    })?;
    server.fn_handler("/favicon.ico", Method::Get, |req| {
        send_file(req, "image/svg+xml", FAVICON_SVG, FAVICON_SVG_GZ)
    })?;

    // 3. GET /api/scan
    let s_scan = Arc::clone(&scanned_ssids);
    server.fn_handler("/api/scan", Method::Get, move |req| {
        let list = s_scan.lock().unwrap();
        let json = serde_json::to_string(&*list).unwrap_or_else(|_| "[]".to_string());
        send_json(req, 200, &json)
    })?;

    // 4. POST /api/provision
    let s_storage = Arc::clone(&storage);
    server.fn_handler("/api/provision", Method::Post, move |mut req| {
        let len = req.content_len().unwrap_or(0) as usize;
        let mut buf = vec![0u8; len];
        let _ = req.read_exact(&mut buf);

        #[derive(serde::Deserialize)]
        struct ProvReq {
            ssid: String,
            password: Option<String>,
        }

        match serde_json::from_slice::<ProvReq>(&buf) {
            Ok(p) => {
                let ssid = p.ssid.trim();
                if ssid.is_empty() {
                    return send_json(req, 400, r#"{"error":"SSID cannot be empty"}"#);
                }
                let pass = p.password.as_deref().unwrap_or("");
                let save_res = {
                    let st = s_storage.lock().unwrap();
                    st.save_wifi(ssid, pass)
                };

                match save_res {
                    Ok(()) => {
                        let res = send_json(
                            req,
                            200,
                            r#"{"status":"success","message":"Credentials saved. Restarting..."}"#,
                        );
                        // Delayed reboot gives HTTP response time to reach browser
                        std::thread::spawn(|| {
                            std::thread::sleep(std::time::Duration::from_millis(1500));
                            unsafe {
                                esp_idf_svc::sys::esp_restart();
                            }
                        });
                        res
                    }
                    Err(e) => {
                        let err_json = format!(r#"{{"error":"{}"}}"#, e);
                        send_json(req, 500, &err_json)
                    }
                }
            }
            Err(_) => send_json(req, 400, r#"{"error":"Invalid JSON payload"}"#),
        }
    })?;

    info!("[HTTP] Provisioning captive portal server listening on http://192.168.4.1:80");
    Ok(server)
}

/// Enables `TCP_NODELAY` on the underlying lwIP socket so HTTP headers and chunks
/// are transmitted immediately without stalling on Nagle + Delayed ACK + Wi-Fi Modem Sleep.
fn set_tcp_nodelay(req: &mut Request<&mut EspHttpConnection>) {
    unsafe {
        let raw_req = req.connection().handle();
        if !raw_req.is_null() {
            let sockfd = esp_idf_svc::sys::httpd_req_to_sockfd(raw_req);
            if sockfd >= 0 {
                let nodelay: core::ffi::c_int = 1;
                let _ = esp_idf_svc::sys::lwip_setsockopt(
                    sockfd,
                    esp_idf_svc::sys::IPPROTO_TCP as core::ffi::c_int,
                    esp_idf_svc::sys::TCP_NODELAY as core::ffi::c_int,
                    &nodelay as *const _ as *const core::ffi::c_void,
                    core::mem::size_of::<core::ffi::c_int>() as esp_idf_svc::sys::socklen_t,
                );
            }
        }
    }
}

fn send_file(
    req: Request<&mut EspHttpConnection>,
    content_type: &str,
    content: &str,
    content_gz: &[u8],
) -> Result<(), EspIOError> {
    send_binary(req, content_type, content.as_bytes(), content_gz)
}

fn send_binary(
    mut req: Request<&mut EspHttpConnection>,
    content_type: &str,
    content: &[u8],
    content_gz: &[u8],
) -> Result<(), EspIOError> {
    crate::led::notify_web_access();
    set_tcp_nodelay(&mut req);

    let accepts_gzip = req
        .header("Accept-Encoding")
        .map(|v| v.contains("gzip"))
        .unwrap_or(false);

    // Allow browsers and reverse proxies to cache static icons for 24 hours to reduce
    // simultaneous connection contention during page load, while keeping HTML/JS/CSS fresh.
    let cache_control = if content_type.starts_with("image/") {
        "public, max-age=86400"
    } else {
        "no-cache"
    };

    if accepts_gzip {
        let headers = [
            ("Content-Type", content_type),
            ("Content-Encoding", "gzip"),
            ("Cache-Control", cache_control),
        ];
        let mut resp = req.into_response(200, Some("OK"), &headers)?;
        resp.write_all(content_gz)?;
    } else {
        let headers = [
            ("Content-Type", content_type),
            ("Cache-Control", cache_control),
        ];
        let mut resp = req.into_response(200, Some("OK"), &headers)?;
        resp.write_all(content)?;
    }
    Ok(())
}

fn send_json(
    mut req: Request<&mut EspHttpConnection>,
    status: u16,
    json: &str,
) -> Result<(), EspIOError> {
    crate::led::notify_web_access();
    set_tcp_nodelay(&mut req);
    let headers = [
        ("Content-Type", "application/json"),
        ("Cache-Control", "no-store"),
        ("Access-Control-Allow-Origin", "*"),
        ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
        ("Access-Control-Allow-Headers", "*"),
    ];
    let mut resp = req.into_response(status, None, &headers)?;
    resp.write_all(json.as_bytes())?;
    Ok(())
}
