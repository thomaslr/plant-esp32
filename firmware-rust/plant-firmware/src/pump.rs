//! Hardware Pump Gate Driver.
//!
//! Controls GPIO 7 (MOSFET Gate).
//! Implements `plant_core::safety::PumpGate`.
//! Note: Status LED (GPIO 8) is controlled independently by the LED Supervisor thread.

use esp_idf_svc::sys::{
    gpio_mode_t_GPIO_MODE_OUTPUT, gpio_reset_pin, gpio_set_direction, gpio_set_level,
};
use log::info;
use plant_core::safety::PumpGate;

use crate::pins::PIN_PUMP_GATE;

pub struct HardwarePumpGate;

impl HardwarePumpGate {
    /// Initializes GPIO 7, ensuring pump is strictly OFF at boot.
    pub fn new() -> Self {
        unsafe {
            // Initialize Pump MOSFET Gate (GPIO 7) - Start LOW
            gpio_reset_pin(PIN_PUMP_GATE);
            gpio_set_direction(PIN_PUMP_GATE, gpio_mode_t_GPIO_MODE_OUTPUT);
            gpio_set_level(PIN_PUMP_GATE, 0);
        }

        info!("[PUMP] Hardware initialized. Pump gate: LOW (OFF)");
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
                // Pump ON (MOSFET Gate HIGH)
                gpio_set_level(PIN_PUMP_GATE, 1);
            } else {
                // Pump OFF (MOSFET Gate LOW)
                gpio_set_level(PIN_PUMP_GATE, 0);
            }
        }
    }
}
