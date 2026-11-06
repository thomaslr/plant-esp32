//! Hardware Pump Gate Driver with 5 kHz LEDC Hardware PWM.
//!
//! Controls GPIO 10 (AO3400A MOSFET Gate) with configurable duty cycle (10..100%).
//! Implements `plant_core::safety::PumpGate`.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use esp_idf_svc::sys::{
    ledc_channel_config, ledc_channel_config_t, ledc_channel_t_LEDC_CHANNEL_0,
    ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_set_duty, ledc_timer_bit_t_LEDC_TIMER_10_BIT,
    ledc_timer_config, ledc_timer_config_t, ledc_timer_t_LEDC_TIMER_0, ledc_update_duty,
};
use log::info;
use plant_core::safety::PumpGate;

use crate::pins::PIN_PUMP_GATE;

pub struct HardwarePumpGate {
    speed_pct: Arc<AtomicU32>,
}

impl HardwarePumpGate {
    /// Initializes 5 kHz LEDC Hardware PWM on GPIO 10, ensuring pump is strictly OFF (0% duty) at boot.
    pub fn new(speed_pct: Arc<AtomicU32>) -> Self {
        unsafe {
            // Configure 5 kHz, 10-bit resolution timer
            let timer_conf = ledc_timer_config_t {
                speed_mode: ledc_mode_t_LEDC_LOW_SPEED_MODE,
                duty_resolution: ledc_timer_bit_t_LEDC_TIMER_10_BIT,
                timer_num: ledc_timer_t_LEDC_TIMER_0,
                freq_hz: 5000,
                clk_cfg: 0,
                deconfigure: false,
            };
            ledc_timer_config(&timer_conf);

            // Configure LEDC Channel 0 on GPIO 10
            let channel_conf = ledc_channel_config_t {
                gpio_num: PIN_PUMP_GATE,
                speed_mode: ledc_mode_t_LEDC_LOW_SPEED_MODE,
                channel: ledc_channel_t_LEDC_CHANNEL_0,
                intr_type: 0,
                timer_sel: ledc_timer_t_LEDC_TIMER_0,
                duty: 0,
                hpoint: 0,
                sleep_mode: 0,
                flags: Default::default(),
            };
            ledc_channel_config(&channel_conf);
        }

        info!(
            "[PUMP] Hardware initialized with 5 kHz PWM on GPIO 10. Default speed: {}%",
            speed_pct.load(Ordering::Relaxed)
        );
        Self { speed_pct }
    }
}

impl Default for HardwarePumpGate {
    fn default() -> Self {
        Self::new(Arc::new(AtomicU32::new(60)))
    }
}

impl PumpGate for HardwarePumpGate {
    fn set_pump(&mut self, on: bool) {
        let duty = if on {
            let pct = self.speed_pct.load(Ordering::Relaxed).clamp(10, 100);
            info!("[PUMP] Gate ENGAGED with 5 kHz PWM at {}% power", pct);
            (pct * 1023) / 100
        } else {
            info!("[PUMP] Gate STOPPED (0% PWM)");
            0
        };

        unsafe {
            ledc_set_duty(
                ledc_mode_t_LEDC_LOW_SPEED_MODE,
                ledc_channel_t_LEDC_CHANNEL_0,
                duty,
            );
            ledc_update_duty(
                ledc_mode_t_LEDC_LOW_SPEED_MODE,
                ledc_channel_t_LEDC_CHANNEL_0,
            );
        }
    }
}
