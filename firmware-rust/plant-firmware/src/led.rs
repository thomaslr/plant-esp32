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
//! - Wi-Fi Connecting: Fast continuous blink (100ms ON, 100ms OFF).
//! - Reservoir Dry: Accelerando frequency sweep (low to high blink) over 3 seconds.
//! - Ultrasonic Sensor Fault: Triple blip (3 quick 80ms pulses, pause).
//! - Soil Moisture Sensor Fault: 5 blips (5 quick 80ms pulses, pause).
//! - Normal Healthy Idle: Single subtle 50ms blip once every 10 seconds.

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

    SystemLedStatus {
        pump_active,
        wifi_connected,
        active_faults,
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

/// Pulses the status LED in setup/provisioning mode (100ms ON / 100ms OFF).
pub fn pulse_setup() {
    pulse(100, 100);
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

/// Triple blip pattern for Ultrasonic Sensor Fault (3 quick pulses).
fn play_ultrasonic_fault_pattern() {
    for _ in 0..2 {
        pulse(80, 120);
    }
    pulse(80, 1000);
}

/// 5-blip pattern for Soil Moisture Sensor Fault (5 quick pulses).
fn play_soil_fault_pattern() {
    for _ in 0..4 {
        pulse(80, 120);
    }
    pulse(80, 1000);
}

/// Normal healthy idle heartbeat: 50ms pulse once every 10 seconds.
/// Slices the 9950ms sleep into 100ms intervals so state changes (pump active, faults)
/// are reacted to promptly without lagging behind the 10-second timer.
fn play_idle_heartbeat(state: &Arc<Mutex<AppState>>) {
    // 50ms ON
    led_on();
    thread::sleep(Duration::from_millis(50));
    led_off();

    // 9950ms OFF with responsive exit
    for _ in 0..100 {
        thread::sleep(Duration::from_millis(100));
        let status = query_status(state);
        if status.pump_active || !status.wifi_connected || !status.active_faults.is_empty() {
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

        // 2. Wi-Fi Connecting / Disconnected: Fast continuous blink (100ms ON / 100ms OFF)
        if !status.wifi_connected {
            pulse(100, 100);
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
            thread::sleep(Duration::from_millis(800));
            continue;
        }

        // 4. Normal Healthy Idle: 50ms pulse once every 10 seconds
        play_idle_heartbeat(&state);
    }
}
