//! Hardware Sensor Acquisition Driver.
//!
//! Controls ADC sampling (supply voltage, battery voltage, soil moisture),
//! switched soil probe power (GPIO 5 duty cycle), and ultrasonic timing (HC-SR04).

use std::sync::atomic::{AtomicU32, Ordering};
use std::thread::sleep;
use std::time::Duration;

use esp_idf_svc::sys::{
    adc1_channel_t, adc1_channel_t_ADC1_CHANNEL_0, adc1_channel_t_ADC1_CHANNEL_1,
    adc1_channel_t_ADC1_CHANNEL_3, adc1_channel_t_ADC1_CHANNEL_4, adc1_config_channel_atten,
    adc1_config_width, adc1_get_raw, adc_atten_t_ADC_ATTEN_DB_11,
    adc_bits_width_t_ADC_WIDTH_BIT_12, adc_unit_t_ADC_UNIT_1, esp_adc_cal_characteristics_t,
    esp_adc_cal_characterize, esp_adc_cal_raw_to_voltage, esp_timer_get_time, gpio_get_level,
    gpio_mode_t_GPIO_MODE_INPUT, gpio_mode_t_GPIO_MODE_OUTPUT, gpio_reset_pin, gpio_set_direction,
    gpio_set_level, soc_periph_temperature_sensor_clk_src_t_TEMPERATURE_SENSOR_CLK_SRC_DEFAULT,
    temperature_sensor_config_t, temperature_sensor_disable, temperature_sensor_enable,
    temperature_sensor_get_celsius, temperature_sensor_handle_t, temperature_sensor_install,
    temperature_sensor_uninstall, ESP_OK,
};
use log::{debug, info, warn};
use plant_core::config::AppConfig;
use plant_core::reading::SensorReading;
use plant_core::sensors::{
    divider_volts, echo_us_to_cm, is_valid_distance, median_f32, median_i32, soil_percent,
    water_metrics,
};

use crate::pins::{
    PIN_BATTERY_MONITOR_ADC, PIN_SOIL_MOISTURE_ADC, PIN_SOIL_POWER, PIN_SUPPLY_MONITOR_ADC,
    PIN_ULTRASONIC_ECHO, PIN_ULTRASONIC_TRIG,
};

#[derive(Clone, Copy)]
struct TempSensorHandle(temperature_sensor_handle_t);
unsafe impl Send for TempSensorHandle {}
unsafe impl Sync for TempSensorHandle {}

pub struct HardwareSensorManager {
    preventive_resets_count: AtomicU32,
    temp_sensor: Option<TempSensorHandle>,
}

impl HardwareSensorManager {
    /// Initializes GPIO pins, ADC1 peripheral with 12-bit resolution and 11dB attenuation,
    /// and the ESP32-C3 internal silicon temperature sensor.
    pub fn new() -> Self {
        unsafe {
            // Configure ADC1 12-bit capture width
            adc1_config_width(adc_bits_width_t_ADC_WIDTH_BIT_12);

            // Configure 11dB attenuation (~0-3.1V range) on all active ADC1 channels:
            // Channel 0 = GPIO 0 (5V Rail Divider)
            // Channel 1 = GPIO 1 (Capacitive Soil Sensor)
            // Channel 3 = GPIO 3 (Ambient Light LDR)
            // Channel 4 = GPIO 4 (18650 Battery Divider)
            adc1_config_channel_atten(adc1_channel_t_ADC1_CHANNEL_0, adc_atten_t_ADC_ATTEN_DB_11);
            adc1_config_channel_atten(adc1_channel_t_ADC1_CHANNEL_1, adc_atten_t_ADC_ATTEN_DB_11);
            adc1_config_channel_atten(adc1_channel_t_ADC1_CHANNEL_3, adc_atten_t_ADC_ATTEN_DB_11);
            adc1_config_channel_atten(adc1_channel_t_ADC1_CHANNEL_4, adc_atten_t_ADC_ATTEN_DB_11);

            // Ultrasonic TRIG: Output, start LOW
            gpio_reset_pin(PIN_ULTRASONIC_TRIG);
            gpio_set_direction(PIN_ULTRASONIC_TRIG, gpio_mode_t_GPIO_MODE_OUTPUT);
            gpio_set_level(PIN_ULTRASONIC_TRIG, 0);

            // Ultrasonic ECHO: Input
            gpio_reset_pin(PIN_ULTRASONIC_ECHO);
            gpio_set_direction(PIN_ULTRASONIC_ECHO, gpio_mode_t_GPIO_MODE_INPUT);

            // Switched Soil Power: Output, start LOW (DUTY-CYCLED to avoid corrosion)
            gpio_reset_pin(PIN_SOIL_POWER);
            gpio_set_direction(PIN_SOIL_POWER, gpio_mode_t_GPIO_MODE_OUTPUT);
            gpio_set_level(PIN_SOIL_POWER, 0);
        }

        let temp_sensor = unsafe {
            let mut handle: temperature_sensor_handle_t = std::ptr::null_mut();
            let mut config = temperature_sensor_config_t::default();
            config.range_min = 10;
            config.range_max = 50;
            config.clk_src = soc_periph_temperature_sensor_clk_src_t_TEMPERATURE_SENSOR_CLK_SRC_DEFAULT;

            let install_res = temperature_sensor_install(&config, &mut handle);
            if install_res == ESP_OK && !handle.is_null() {
                let enable_res = temperature_sensor_enable(handle);
                if enable_res == ESP_OK {
                    info!("[SENSOR] Internal ESP32-C3 temperature sensor initialized successfully");
                    Some(TempSensorHandle(handle))
                } else {
                    warn!("[SENSOR] Failed to enable temperature sensor: {}", enable_res);
                    temperature_sensor_uninstall(handle);
                    None
                }
            } else {
                warn!("[SENSOR] Failed to install temperature sensor: {}", install_res);
                None
            }
        };

        Self {
            preventive_resets_count: AtomicU32::new(0),
            temp_sensor,
        }
    }

    /// Preventive reset for stuck HC-SR04 clone sensors.
    ///
    /// Drives the ECHO pin LOW as an output for 10µs to clear any floating latch,
    /// then restores to high-impedance INPUT and pauses 30ms for acoustic damping.
    pub fn force_ultrasonic_reset(&self) {
        unsafe {
            gpio_set_direction(PIN_ULTRASONIC_ECHO, gpio_mode_t_GPIO_MODE_OUTPUT);
            gpio_set_level(PIN_ULTRASONIC_ECHO, 0);
            esp_idf_svc::sys::esp_rom_delay_us(10);
            gpio_set_direction(PIN_ULTRASONIC_ECHO, gpio_mode_t_GPIO_MODE_INPUT);
        }
        sleep(Duration::from_millis(30));
        self.preventive_resets_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Measures ultrasonic distance in centimeters with multi-sample median filtering
    /// and hard timeout recovery.
    pub fn read_ultrasonic_distance_cm(&self) -> f32 {
        let mut samples = [0.0f32; 3];
        let mut valid_samples = 0;

        for _ in 0..3 {
            unsafe {
                // If ECHO pin is stuck HIGH before trigger, clear immediately
                if gpio_get_level(PIN_ULTRASONIC_ECHO) == 1 {
                    self.force_ultrasonic_reset();
                }

                // Send 10µs TRIG pulse
                gpio_set_level(PIN_ULTRASONIC_TRIG, 0);
                esp_idf_svc::sys::esp_rom_delay_us(2);
                gpio_set_level(PIN_ULTRASONIC_TRIG, 1);
                esp_idf_svc::sys::esp_rom_delay_us(10);
                gpio_set_level(PIN_ULTRASONIC_TRIG, 0);

                // Wait for ECHO pin to go HIGH (max 5ms wait)
                let t_trig = esp_timer_get_time();
                let mut started = false;
                while (esp_timer_get_time() - t_trig) < 5000 {
                    if gpio_get_level(PIN_ULTRASONIC_ECHO) == 1 {
                        started = true;
                        break;
                    }
                }

                if !started {
                    sleep(Duration::from_millis(10));
                    continue;
                }

                // Measure ECHO HIGH duration with a HARD 30ms timeout
                let t_start = esp_timer_get_time();
                let mut ended = false;
                while (esp_timer_get_time() - t_start) < 30_000 {
                    if gpio_get_level(PIN_ULTRASONIC_ECHO) == 0 {
                        ended = true;
                        break;
                    }
                }
                let t_end = esp_timer_get_time();

                if !ended {
                    // Echo timed out: stuck latch or acoustic lost -> reset sensor
                    self.force_ultrasonic_reset();
                    continue;
                }

                let duration_us = t_end - t_start;
                let distance = echo_us_to_cm(duration_us);

                if is_valid_distance(distance) {
                    samples[valid_samples] = distance;
                    valid_samples += 1;
                }
            }

            sleep(Duration::from_millis(20));
        }

        if valid_samples == 0 {
            -1.0
        } else {
            median_f32(&mut samples[..valid_samples]).unwrap_or(-1.0)
        }
    }

    /// Reads millivolts on a given ADC pin using ESP-IDF ADC driver.
    ///
    /// Samples 5 times with 5ms spacing and returns the median millivolts.
    pub fn read_adc_mv(&self, pin: i32) -> i32 {
        let mut samples = [0i32; 5];
        for i in 0..5 {
            samples[i] = read_raw_channel_mv(pin);
            sleep(Duration::from_millis(5));
        }
        median_i32(&mut samples).unwrap_or(0)
    }

    /// Powers on the capacitive soil sensor probe for 30ms, samples ADC, and powers it OFF.
    pub fn read_soil_moisture_raw(&self) -> i32 {
        unsafe {
            // 1. Power ON probe
            gpio_set_level(PIN_SOIL_POWER, 1);
        }

        // 2. Allow 30ms for 555 oscillator and bypass filter capacitor to stabilize
        sleep(Duration::from_millis(30));

        let raw_mv = self.read_adc_mv(PIN_SOIL_MOISTURE_ADC);

        unsafe {
            // 3. Power OFF immediately to eliminate idle draw and galvanic corrosion
            gpio_set_level(PIN_SOIL_POWER, 0);
        }

        raw_mv
    }

    /// Reads supply voltage (5V rail) via 100k/100k divider.
    pub fn read_supply_voltage(&self) -> f32 {
        divider_volts(self.read_adc_mv(PIN_SUPPLY_MONITOR_ADC))
    }

    /// Reads 18650 battery voltage via divider (returns 0.0V if unpopulated).
    pub fn read_battery_voltage(&self) -> f32 {
        if PIN_BATTERY_MONITOR_ADC < 0 {
            0.0
        } else {
            divider_volts(self.read_adc_mv(PIN_BATTERY_MONITOR_ADC))
        }
    }

    /// Reads ESP32-C3 internal silicon temperature in Celsius.
    pub fn read_chip_temperature(&self) -> f32 {
        if let Some(sensor) = self.temp_sensor {
            let mut temp = 0.0f32;
            let res = unsafe { temperature_sensor_get_celsius(sensor.0, &mut temp) };
            if res == ESP_OK {
                return (temp * 10.0).round() / 10.0;
            } else {
                warn!("[SENSOR] Failed to read internal chip temperature: {}", res);
            }
        }
        0.0
    }

    /// Returns the number of times the ultrasonic stuck-latch preventive reset fired.
    pub fn preventive_resets_count(&self) -> u32 {
        self.preventive_resets_count.load(Ordering::Relaxed)
    }

    /// Reads all hardware sensors and compiles a `SensorReading` struct.
    pub fn read_all_sensors(&self, pump_active: bool, config: &AppConfig) -> SensorReading {
        let now_sec = unsafe { (esp_timer_get_time() / 1_000_000) as u32 };

        let supply_voltage = self.read_supply_voltage();
        let battery_voltage = self.read_battery_voltage();
        let chip_temp = self.read_chip_temperature();

        let soil_raw_adc = self.read_soil_moisture_raw();
        let soil_percent = soil_percent(
            soil_raw_adc,
            config.gauges.soil_moisture.dry_adc,
            config.gauges.soil_moisture.wet_adc,
        );

        let distance_cm = self.read_ultrasonic_distance_cm();
        let metrics = water_metrics(
            distance_cm,
            config.gauges.water_level.tank_height_cm,
            config.gauges.water_level.sensor_offset_cm,
        );

        debug!(
            "[SENSOR] Soil: {:.1}% ({}mV) | Water: {:.1}cm | 5V: {:.2}V | Bat: {:.2}V | Chip: {:.1}°C",
            soil_percent, soil_raw_adc, metrics.level_cm, supply_voltage, battery_voltage, chip_temp
        );

        SensorReading {
            timestamp: now_sec,
            pump_active,
            supply_voltage,
            battery_voltage,
            chip_temp,
            soil_raw_adc,
            soil_percent,
            distance_cm,
            water_level_cm: metrics.level_cm,
            water_percent: metrics.percent,
        }
    }
}

impl Default for HardwareSensorManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to read factory eFuse-calibrated millivolts on ESP32-C3 ADC1 channels.
fn read_raw_channel_mv(pin: i32) -> i32 {
    let channel: adc1_channel_t = match pin {
        0 => adc1_channel_t_ADC1_CHANNEL_0,
        1 => adc1_channel_t_ADC1_CHANNEL_1,
        3 => adc1_channel_t_ADC1_CHANNEL_3,
        4 => adc1_channel_t_ADC1_CHANNEL_4,
        _ => adc1_channel_t_ADC1_CHANNEL_0,
    };

    unsafe {
        let raw = adc1_get_raw(channel);
        if raw >= 0 {
            let mut chars = std::mem::zeroed::<esp_adc_cal_characteristics_t>();
            esp_adc_cal_characterize(
                adc_unit_t_ADC_UNIT_1,
                adc_atten_t_ADC_ATTEN_DB_11,
                adc_bits_width_t_ADC_WIDTH_BIT_12,
                1100,
                &mut chars,
            );
            esp_adc_cal_raw_to_voltage(raw as u32, &chars) as i32
        } else {
            0
        }
    }
}

impl Drop for HardwareSensorManager {
    fn drop(&mut self) {
        if let Some(sensor) = self.temp_sensor {
            unsafe {
                let _ = temperature_sensor_disable(sensor.0);
                let _ = temperature_sensor_uninstall(sensor.0);
            }
        }
    }
}
