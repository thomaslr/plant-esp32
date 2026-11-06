//! Onboard Status LED Supervisor (ESP32-C3 SuperMini GPIO 8).
//!
//! Provides deterministic visual feedback with zero race conditions and zero deadlocks:
//! - Exclusive single-owner control of `PIN_STATUS_LED` (Active LOW: 0 = ON, 1 = OFF).
//! - Short lock acquisitions (< 2µs) on `AppState` to prevent contention or blocking.
//! - Approach 2 (Round-Robin Alternating): when multiple faults are active simultaneously,
//!   the supervisor rotates sequentially through all active fault patterns so none are masked.
//!
//! Visual Signaling Patterns:
//! - Watering Active: Solid ON while pump is running.
//! - Remote Web Access: Quick double-pulse flicker (40ms ON, 60ms OFF, 40ms ON, 400ms pause) on HTTP activity.
//! - Wi-Fi Connecting: Distinct beacon blink (200ms ON, 800ms OFF).
//! - Reservoir Dry: Accelerando frequency sweep (low to high blink) over 3 seconds.
//! - Ultrasonic Sensor Fault: 4 blips (4 pulses: 160ms ON, 240ms OFF, 2000ms pause).
//! - Soil Moisture Sensor Fault: Triple blip (3 pulses: 160ms ON, 240ms OFF, 2000ms pause).
//! - Normal Healthy Idle: Single calm 100ms blip once every 10 seconds.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use esp_idf_svc::sys::{
    gpio_mode_t_GPIO_MODE_OUTPUT, gpio_reset_pin, gpio_set_direction, gpio_set_level,
};
use log::info;
use plant_core::sensors::{is_valid_distance, is_valid_soil_raw};

use crate::pins::PIN_STATUS_LED;
use crate::web::AppState;

/// Shared millisecond timestamp of the most recent HTTP webserver request.
pub static LAST_HTTP_ACTIVITY_MS: AtomicU32 = AtomicU32::new(0);

/// Notifies the visual signaling system of an incoming web request.
pub fn notify_web_access() {
    let now_ms = unsafe { (esp_idf_svc::sys::esp_timer_get_time() / 1000) as u32 };
    LAST_HTTP_ACTIVITY_MS.store(now_ms, Ordering::Relaxed);
}

/// Active fault categories monitored by the visual signaling supervisor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultType {
    /// Reservoir water level is below minimum safe cutoff with a valid ultrasonic reading.
    ReservoirDry,
    /// Ultrasonic sensor returned invalid distance, echo timeout, or stuck latch.
    UltrasonicFault,
    /// Capacitive soil sensor voltage is outside plausible range (< 300mV or > 3200mV).
    SoilFault,
}

/// Instantaneous status snapshot for LED state evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct SystemLedStatus {
    pub pump_active: bool,
    pub wifi_connected: bool,
    pub active_faults: Vec<FaultType>,
    pub web_active: bool,
}

#[inline]
fn led_on() {
    unsafe {
        gpio_set_level(PIN_STATUS_LED, 0); // Active LOW: 0 = ON
    }
}

#[inline]
fn led_off() {
    unsafe {
        gpio_set_level(PIN_STATUS_LED, 1); // Active LOW: 1 = OFF
    }
}

fn pulse(on_ms: u64, off_ms: u64) {
    led_on();
    thread::sleep(Duration::from_millis(on_ms));
    led_off();
    thread::sleep(Duration::from_millis(off_ms));
}

/// Takes a sub-microsecond snapshot of system state from `AppState`.
pub fn query_status(state: &Arc<Mutex<AppState>>) -> SystemLedStatus {
    let app = state.lock().unwrap();
    let pump_active = app.safety.is_pump_active();
    let wifi_connected = app.ip_address != "0.0.0.0";
    let min_safe_water = app.config.gauges.water_level.min_safe_level_cm;

    let mut active_faults = Vec::with_capacity(3);

    if let Some(reading) = app.history.latest() {
        // 1. Ultrasonic Sensor Fault Check
        let ultrasonic_fault = !is_valid_distance(reading.distance_cm);
        if ultrasonic_fault {
            active_faults.push(FaultType::UltrasonicFault);
        }

        // 2. Reservoir Dry Check (only when ultrasonic sensor is healthy)
        if !ultrasonic_fault && reading.water_level_cm < min_safe_water {
            active_faults.push(FaultType::ReservoirDry);
        }

        // 3. Capacitive Soil Probe Fault Check
        if !is_valid_soil_raw(reading.soil_raw_adc) {
            active_faults.push(FaultType::SoilFault);
        }
    }

    let now_ms = unsafe { (esp_idf_svc::sys::esp_timer_get_time() / 1000) as u32 };
    let last_http = LAST_HTTP_ACTIVITY_MS.load(Ordering::Relaxed);
    let web_active = last_http > 0 && now_ms.saturating_sub(last_http) < 2500;

    SystemLedStatus {
        pump_active,
        wifi_connected,
        active_faults,
        web_active,
    }
}

/// Initializes GPIO 8 as output and forces the LED strictly OFF.
pub fn init_led_hardware() {
    unsafe {
        gpio_reset_pin(PIN_STATUS_LED);
        gpio_set_direction(PIN_STATUS_LED, gpio_mode_t_GPIO_MODE_OUTPUT);
        gpio_set_level(PIN_STATUS_LED, 1); // Active LOW: 1 = OFF
    }
    info!("[LED] Status LED initialized on GPIO 8 (Active LOW, initial: OFF)");
}

/// Pulses the status LED in setup/provisioning mode (200ms ON / 800ms OFF).
pub fn pulse_setup() {
    pulse(200, 800);
}

/// Accelerando frequency sweep (low to high blink) over 3 seconds for Reservoir Dry.
fn play_reservoir_dry_pattern() {
    // 8 pulses with progressively decreasing periods (from 300ms down to 25ms)
    // Sum of pulses: 600 + 480 + 360 + 260 + 180 + 120 + 80 + 50 = 2130 ms.
    // Followed by 870 ms OFF pause = exactly 3000 ms total.
    const ACCEL_TIMINGS: &[(u64, u64)] = &[
        (300, 300),
        (240, 240),
        (180, 180),
        (130, 130),
        (90, 90),
        (60, 60),
        (40, 40),
        (25, 25),
    ];

    for &(on_ms, off_ms) in ACCEL_TIMINGS {
        pulse(on_ms, off_ms);
    }
    thread::sleep(Duration::from_millis(870));
}

/// 4-blip pattern for Ultrasonic Sensor Fault (4 pulses: 160ms ON / 240ms OFF).
fn play_ultrasonic_fault_pattern() {
    for _ in 0..3 {
        pulse(160, 240);
    }
    pulse(160, 2000);
}

/// Triple blip pattern for Soil Moisture Sensor Fault (3 pulses: 160ms ON / 240ms OFF).
fn play_soil_fault_pattern() {
    for _ in 0..2 {
        pulse(160, 240);
    }
    pulse(160, 2000);
}

/// Normal healthy idle heartbeat: 100ms pulse once every 10 seconds.
/// Slices the 9900ms sleep into 100ms intervals so state changes (pump active, faults, web traffic)
/// are reacted to promptly without lagging behind the 10-second timer.
fn play_idle_heartbeat(state: &Arc<Mutex<AppState>>) {
    // 100ms ON (doubled from 50ms for a calmer, clearly visible pulse)
    led_on();
    thread::sleep(Duration::from_millis(100));
    led_off();

    // 9900ms OFF with responsive exit
    for _ in 0..99 {
        thread::sleep(Duration::from_millis(100));
        if crate::ota::is_ota_in_progress() {
            break;
        }
        let status = query_status(state);
        if status.pump_active || !status.wifi_connected || !status.active_faults.is_empty() || status.web_active {
            break;
        }
    }
}

/// Main entry point for the dedicated LED supervisor background task.
pub fn run_led_supervisor(state: Arc<Mutex<AppState>>) -> ! {
    init_led_hardware();
    let mut rr_index: usize = 0;

    info!("[LED] LED Supervisor started (Approach 2: Round-Robin multi-fault rotation)");

    loop {
        // 0. Super-Priority: Firmware OTA Flashing Active (Fast 10 Hz strobe: 50ms ON / 50ms OFF)
        if crate::ota::is_ota_in_progress() {
            pulse(50, 50);
            continue;
        }

        let status = query_status(&state);

        // 1. Highest Priority: Watering Active (Solid ON while pump runs)
        if status.pump_active {
            led_on();
            while {
                let app = state.lock().unwrap();
                app.safety.is_pump_active()
            } {
                thread::sleep(Duration::from_millis(50));
            }
            led_off();
            thread::sleep(Duration::from_millis(100));
            continue;
        }

        // 2. Wi-Fi Connecting / Disconnected: Distinct beacon blink (200ms ON / 800ms OFF)
        if !status.wifi_connected {
            pulse(200, 800);
            continue;
        }

        // 3. Active Faults: Round-Robin alternation
        if !status.active_faults.is_empty() {
            let fault = status.active_faults[rr_index % status.active_faults.len()];
            rr_index = rr_index.wrapping_add(1);

            match fault {
                FaultType::ReservoirDry => {
                    play_reservoir_dry_pattern();
                }
                FaultType::UltrasonicFault => {
                    play_ultrasonic_fault_pattern();
                }
                FaultType::SoilFault => {
                    play_soil_fault_pattern();
                }
            }

            // Separator pause between alternating fault patterns
            thread::sleep(Duration::from_millis(1600));
            continue;
        }

        // 4. Web Traffic Activity (Remote user browsing/interacting with Web UI)
        // Distinct fast double-tap flicker (40ms ON, 60ms OFF, 40ms ON, 400ms pause)
        if status.web_active {
            pulse(40, 60);
            pulse(40, 400);
            continue;
        }

        // 5. Normal Healthy Idle: 100ms pulse once every 10 seconds
        play_idle_heartbeat(&state);
    }
}
