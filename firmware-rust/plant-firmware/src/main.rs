//! Main entry point for the Plant ESP32-C3 Firmware in Rust.
//!
//! Replaces the PlatformIO / Arduino / FreeRTOS C++ firmware.
//! Spawns four concurrent tasks:
//! 1. Safety Supervisor Task (20 Hz / 50ms): Evaluates runtime limits, dry tank cutoffs, and cooldowns.
//! 2. Sensor Acquisition Task: Reads ADC voltages, soil moisture (duty-cycled), and ultrasonic level.
//! 3. Network & Time Supervisor: Maintains Wi-Fi, synchronizes NTP time, and resets daily watering quota at midnight.
//! 4. Visual Status LED Supervisor: Dedicated Round-Robin status signaling on GPIO 8.

mod led;
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
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sntp::EspSntp;
use esp_idf_svc::sys::{esp_timer_get_time, link_patches};
use log::{info, warn};
use plant_core::ring_buffer::RingBuffer;
use plant_core::safety::SafetyManager;
use plant_core::timeutil::{local_day, local_hour, MidnightTracker};

use crate::led::run_led_supervisor;
use crate::pump::HardwarePumpGate;
use crate::secrets::{WIFI_PASSWORD, WIFI_SSID};
use crate::sensors::HardwareSensorManager;
use crate::storage::NvsConfigStorage;
use crate::web::{create_web_server, AppState};
use crate::wifi::init_wifi;

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

    // 4. Initialize Hardware Pump Gate and Safety Supervisor
    let pump_gate = HardwarePumpGate::new();
    let safety = SafetyManager::new(&config, pump_gate);

    // 5. Initialize Sensor Manager & 100-sample RAM Ring Buffer
    let sensors = HardwareSensorManager::new();
    let history = RingBuffer::new(100);

    // 6. Connect to Wi-Fi with Modem Sleep enabled
    let wifi_handle = match init_wifi(
        peripherals.modem,
        sys_loop,
        Some(nvs),
        WIFI_SSID,
        WIFI_PASSWORD,
        &config.system.hostname,
    ) {
        Ok(w) => Some(w),
        Err(e) => {
            warn!("[WIFI] Failed to initialize Wi-Fi: {:?}", e);
            None
        }
    };

    let ip_str = if let Some(ref w) = wifi_handle {
        w.wifi()
            .sta_netif()
            .get_ip_info()
            .map(|i| i.ip.to_string())
            .unwrap_or_else(|_| "0.0.0.0".to_string())
    } else {
        "0.0.0.0".to_string()
    };

    // 7. Start SNTP Time Synchronization
    let sntp = EspSntp::new_default();
    if sntp.is_ok() {
        info!("[NTP] SNTP client initialized for time synchronization");
    }

    // 8. Construct shared application state
    let state = Arc::new(Mutex::new(AppState {
        config: config.clone(),
        safety,
        sensors,
        history,
        storage,
        ip_address: ip_str,
        wifi_rssi: -60,
    }));

    // 9. Start Embedded HTTP Web Server
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

    // Prevent server and wifi from being dropped
    if let Some(srv) = _server {
        core::mem::forget(srv);
    }
    if let Some(w) = wifi_handle {
        core::mem::forget(w);
    }

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

    // Keep main thread alive
    loop {
        thread::sleep(Duration::from_secs(3600));
    }
}
