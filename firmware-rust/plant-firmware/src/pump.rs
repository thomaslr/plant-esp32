//! Hardware Pump Gate and Status LED Driver.
//!
//! Controls GPIO 7 (MOSFET Gate) and GPIO 8 (Active-LOW Status LED).
//! Implements `plant_core::safety::PumpGate`.

use esp_idf_svc::sys::{
    gpio_mode_t_GPIO_MODE_OUTPUT, gpio_reset_pin, gpio_set_direction, gpio_set_level,
};
use log::info;
use plant_core::safety::PumpGate;

use crate::pins::{PIN_PUMP_GATE, PIN_STATUS_LED};

pub struct HardwarePumpGate;

impl HardwarePumpGate {
    /// Initializes GPIO 7 and GPIO 8, ensuring pump and LED are strictly OFF at boot.
    pub fn new() -> Self {
        unsafe {
            // Initialize Pump MOSFET Gate (GPIO 7) - Start LOW
            gpio_reset_pin(PIN_PUMP_GATE);
            gpio_set_direction(PIN_PUMP_GATE, gpio_mode_t_GPIO_MODE_OUTPUT);
            gpio_set_level(PIN_PUMP_GATE, 0);

            // Initialize Status LED (GPIO 8) - Active LOW: Start HIGH (OFF)
            gpio_reset_pin(PIN_STATUS_LED);
            gpio_set_direction(PIN_STATUS_LED, gpio_mode_t_GPIO_MODE_OUTPUT);
            gpio_set_level(PIN_STATUS_LED, 1);
        }

        info!("[PUMP] Hardware initialized. Pump gate: LOW (OFF), Status LED: HIGH (OFF)");
        Self
    }
}

impl Default for HardwarePumpGate {
    fn default() -> Self {
        Self::new()
    }
}

impl PumpGate for HardwarePumpGate {
    fn set_pump(&mut self, on: bool) {
        unsafe {
            if on {
                // Pump ON (MOSFET Gate HIGH), Blue LED ON (Active LOW = 0)
                gpio_set_level(PIN_PUMP_GATE, 1);
                gpio_set_level(PIN_STATUS_LED, 0);
            } else {
                // Pump OFF (MOSFET Gate LOW), Blue LED OFF (Active LOW = 1)
                gpio_set_level(PIN_PUMP_GATE, 0);
                gpio_set_level(PIN_STATUS_LED, 1);
            }
        }
    }
}
