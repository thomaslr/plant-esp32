//! Main entry point for the Plant ESP32-C3 Firmware in Rust.
//!
//! Replaces the PlatformIO / Arduino / FreeRTOS C++ firmware.
//! Spawns four concurrent tasks:
//! 1. Safety Supervisor Task (20 Hz / 50ms): Evaluates runtime limits, dry tank cutoffs, and cooldowns.
//! 2. Sensor Acquisition Task: Reads ADC voltages, soil moisture (duty-cycled), and ultrasonic level.
//! 3. Network & Time Supervisor: Maintains Wi-Fi, synchronizes NTP time, and resets daily watering quota at midnight.
//! 4. Visual Status LED Supervisor: Dedicated Round-Robin status signaling on GPIO 8.

mod led;
mod ota;
mod pins;
mod pump;
mod secrets;
mod sensors;
mod storage;
mod web;
mod wifi;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::mdns::EspMdns;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sntp::EspSntp;
use esp_idf_svc::sys::{esp_timer_get_time, link_patches};
use log::{info, warn};
use plant_core::ring_buffer::RingBuffer;
use plant_core::safety::SafetyManager;
use plant_core::timeutil::{local_day, local_hour, MidnightTracker};

use crate::led::{init_led_hardware, pulse_setup, run_led_supervisor};
use crate::pump::HardwarePumpGate;
use crate::secrets::{WIFI_PASSWORD, WIFI_SSID};
use crate::sensors::HardwareSensorManager;
use crate::storage::{NvsConfigStorage, WifiCredentials};
use crate::web::{create_provisioning_server, create_web_server, AppState};
use crate::wifi::{start_wifi, WifiState};

fn main() -> anyhow::Result<()> {
    // 1. Link ESP-IDF C patches and initialize logger
    link_patches();
    EspLogger::initialize_default();

    info!("============================================");
    info!("   Plant Monitor ESP32-C3 (Rust Edition)");
    info!("============================================");

    // 2. Hardware Peripherals, System Event Loop, and NVS Partition
    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // 3. Load persistent configuration from NVS flash
    let storage = NvsConfigStorage::new(nvs.clone())?;
    let config = storage.load();

    // 4. Resolve saved Wi-Fi credentials (NVS first, secrets.rs developer fallback second)
    let saved_wifi = storage.load_wifi().or_else(|| {
        if !WIFI_SSID.is_empty() && WIFI_SSID != "YOUR_WIFI_SSID" {
            info!("[WIFI] Using developer credentials from secrets.rs");
            Some(WifiCredentials {
                ssid: WIFI_SSID.to_string(),
                password: WIFI_PASSWORD.to_string(),
            })
        } else {
            None
        }
    });

    // 5. Connect to Wi-Fi (Station Mode) or launch Captive Portal (Provisioning SoftAP)
    let wifi_state = start_wifi(
        peripherals.modem,
        sys_loop,
        Some(nvs),
        saved_wifi,
        &config.system.hostname,
    )?;

    match wifi_state {
        WifiState::ProvisioningMode {
            ap_ssid,
            scanned_ssids,
            wifi_handle,
        } => {
            info!("============================================");
            info!("   PROVISIONING SOFTAP ACTIVE");
            info!("   1. Connect your phone to: '{}'", ap_ssid);
            info!("   2. Open browser to: http://192.168.4.1");
            info!("============================================");

            let s_storage = Arc::new(Mutex::new(storage));
            let s_ssids = Arc::new(Mutex::new(scanned_ssids));
            let srv = create_provisioning_server(s_storage, s_ssids)?;
            core::mem::forget(srv);
            core::mem::forget(wifi_handle);

            // Fast LED blink (100ms ON / 100ms OFF) to indicate setup mode
            init_led_hardware();
            loop {
                pulse_setup();
            }
        }
        WifiState::StationConnected {
            ip_address,
            wifi_handle,
        } => {
            // 6. Initialize Hardware Pump Gate and Safety Supervisor
            let pump_gate = HardwarePumpGate::new();
            let safety = SafetyManager::new(&config, pump_gate);

            // 7. Initialize Sensor Manager & 100-sample RAM Ring Buffer
            let sensors = HardwareSensorManager::new();
            let history = RingBuffer::new(100);

            // 8. Start SNTP Time Synchronization
            let sntp = EspSntp::new_default();
            if sntp.is_ok() {
                info!("[NTP] SNTP client initialized for time synchronization");
            }

            // 9. Initialize mDNS Responder (e.g. plants.local or plants1.local)
            let _mdns = match EspMdns::take() {
                Ok(mut mdns) => {
                    let host = &config.system.hostname;
                    if let Err(e) = mdns.set_hostname(host) {
                        warn!("[mDNS] Failed to set hostname '{}': {:?}", host, e);
                    } else {
                        info!("[mDNS] Responder active at http://{}.local", host);
                    }
                    let _ = mdns.set_instance_name(&format!("Plant Monitor ({})", host));
                    let _ = mdns.add_service(None, "_http", "_tcp", 80, &[("board", "esp32c3")]);
                    Some(mdns)
                }
                Err(e) => {
                    warn!("[mDNS] Failed to initialize mDNS: {:?}", e);
                    None
                }
            };

            // 10. Construct shared application state
            let state = Arc::new(Mutex::new(AppState {
                config: config.clone(),
                safety,
                sensors,
                history,
                storage,
                ip_address,
                wifi_rssi: -60,
            }));

            // 11. Start Embedded HTTP Web Server
            let _server = match create_web_server(Arc::clone(&state)) {
                Ok(srv) => {
                    info!("[SYS] HTTP server and REST endpoints listening on port 80");
                    Some(srv)
                }
                Err(e) => {
                    warn!("[SYS] Failed to bind HTTP server: {:?}", e);
                    None
                }
            };

            // Prevent server, mdns and wifi from being dropped
            if let Some(srv) = _server {
                core::mem::forget(srv);
            }
            if let Some(mdns) = _mdns {
                core::mem::forget(mdns);
            }
            core::mem::forget(wifi_handle);

    // 10. Task 1: Safety & Pump Supervisor (Priority: HIGH, 20 Hz / 50ms)
    let s_safety = Arc::clone(&state);
    thread::Builder::new()
        .name("SafetySupervisor".to_string())
        .stack_size(4096)
        .spawn(move || loop {
            let now_ms = unsafe { (esp_timer_get_time() / 1000) as u32 };
            {
                let mut app = s_safety.lock().unwrap();
                app.safety.tick(now_ms);
            }
            thread::sleep(Duration::from_millis(50));
        })?;

    // 11. Task 2: Sensor Acquisition & Auto-Watering (Priority: NORMAL)
    let s_sensors = Arc::clone(&state);
    let startup_delay_sec = config.timing.startup_delay_sec;
    thread::Builder::new()
        .name("SensorAcquisition".to_string())
        .stack_size(8192)
        .spawn(move || {
            // Initial startup delay to let power supplies and sensors stabilize
            info!(
                "[SENSOR] Startup settle delay: {} seconds",
                startup_delay_sec
            );
            thread::sleep(Duration::from_secs(startup_delay_sec as u64));

            loop {
                let (pump_active, config, interval_sec) = {
                    let app = s_sensors.lock().unwrap();
                    (
                        app.safety.is_pump_active(),
                        app.config.clone(),
                        app.config.timing.sensor_read_interval_sec,
                    )
                };

                // Read all hardware sensors
                let reading = {
                    let app = s_sensors.lock().unwrap();
                    app.sensors.read_all_sensors(pump_active, &config)
                };

                // Push reading into circular RAM ring buffer
                {
                    let mut app = s_sensors.lock().unwrap();
                    app.history.push(reading.clone());
                }

                info!(
                    "[SENSOR] Soil: {:.1}% | Water: {:.1}cm | 5V Rail: {:.2}V | Bat: {:.2}V | Chip: {:.1}°C",
                    reading.soil_percent,
                    reading.water_level_cm,
                    reading.supply_voltage,
                    reading.battery_voltage,
                    reading.chip_temp
                );

                // Automated Watering Logic
                if !pump_active {
                    let soil = reading.soil_percent;
                    let threshold = config.gauges.soil_moisture.trigger_threshold as f32;
                    let min_water = config.gauges.water_level.min_safe_level_cm;

                    if soil < threshold && reading.water_level_cm >= min_water {
                        let duration = config.timing.automated_water_duration_sec;
                        let now_sec = unsafe { (esp_timer_get_time() / 1_000_000) as u32 };
                        let now_ms = unsafe { (esp_timer_get_time() / 1000) as u32 };
                        let hour = local_hour(now_sec as i64, config.system.gmt_offset);

                        let mut app = s_sensors.lock().unwrap();
                        let result = app.safety.request_watering(
                            now_ms,
                            duration,
                            "Soil dry trigger",
                            reading.water_level_cm,
                            hour,
                        );

                        if let Ok(actual_duration) = result {
                            info!(
                                "[AUTO] Low soil moisture ({:.1}% < {:.0}%). Started watering for {}s",
                                soil, threshold, actual_duration
                            );
                        }
                    }
                }

                let safe_interval = if interval_sec < 5 { 5 } else { interval_sec };
                thread::sleep(Duration::from_secs(safe_interval as u64));
            }
        })?;

    // 12. Task 3: Midnight Daily Quota Reset & System Monitor (Priority: LOW)
    let s_monitor = Arc::clone(&state);
    thread::Builder::new()
        .name("MidnightTracker".to_string())
        .stack_size(4096)
        .spawn(move || {
            let mut tracker = MidnightTracker::new();
            loop {
                thread::sleep(Duration::from_secs(60));

                let (gmt_offset, now_sec) = {
                    let app = s_monitor.lock().unwrap();
                    let sec = unsafe { (esp_timer_get_time() / 1_000_000) as u32 };
                    (app.config.system.gmt_offset, sec)
                };

                let day = local_day(now_sec as i64, gmt_offset);
                if tracker.update(day) {
                    info!("[NTP] Midnight reached: resetting daily watering quota");
                    let mut app = s_monitor.lock().unwrap();
                    app.safety.reset_daily_quota();
                }
            }
        })?;

    // 13. Task 4: Visual Status LED Supervisor (Priority: NORMAL, Round-Robin Faults)
    let s_led = Arc::clone(&state);
    thread::Builder::new()
        .name("LedSupervisor".to_string())
        .stack_size(4096)
        .spawn(move || {
            run_led_supervisor(s_led);
        })?;

    info!("[SYS] All background tasks successfully spawned. System running.");

    // 14. Confirm successful boot to bootloader to permanently cancel OTA rollback
    crate::ota::mark_running_app_valid();

    // Keep main thread alive
            loop {
                thread::sleep(Duration::from_secs(3600));
            }
        }
    }
}
