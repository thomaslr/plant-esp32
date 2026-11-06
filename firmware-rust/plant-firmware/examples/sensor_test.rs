//! Comprehensive Hardware Diagnostic: Live Soil Moisture & Ultrasonic Distance Telemetry
//!
//! Canonical Pinout Reference:
//! https://lastminuteengineers.com/esp32-c3-super-mini-pinout-reference/
//!
//! Pin Assignments:
//! - Soil Sensor DATA:     GPIO 3  (TB2 Pin 1, ADC1_CH3)
//! - Soil Sensor Power:    GPIO 21 (TB2 Pin 2, Switched 3.3V VCC)
//! - Ultrasonic TRIG:      GPIO 7  (TB1 Pin 2, 10µs pulse out)
//! - Ultrasonic ECHO:      GPIO 20 (TB1 Pin 3, 3.3V-stepped pulse in)
//! - 5V Supply Rail Sense: GPIO 4  (ADC1_CH4 via R1/R2 100k/100k divider)
//! - Status LED:           GPIO 8  (Onboard Blue LED, Active LOW)

use std::thread::sleep;
use std::time::Duration;

use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::{
    adc1_channel_t_ADC1_CHANNEL_3, adc1_channel_t_ADC1_CHANNEL_4, adc1_config_channel_atten,
    adc1_config_width, adc1_get_raw, adc_atten_t_ADC_ATTEN_DB_11,
    adc_bits_width_t_ADC_WIDTH_BIT_12, adc_unit_t_ADC_UNIT_1, esp_adc_cal_characteristics_t,
    esp_adc_cal_characterize, esp_adc_cal_raw_to_voltage, esp_rom_delay_us, esp_timer_get_time,
    gpio_get_level, gpio_mode_t_GPIO_MODE_INPUT, gpio_mode_t_GPIO_MODE_OUTPUT, gpio_reset_pin,
    gpio_set_direction, gpio_set_level, link_patches,
};
use plant_core::sensors::{echo_us_to_cm, is_valid_distance, median_f32, soil_percent};

// Canonical GPIO mappings
const _PIN_SOIL_DATA: i32 = 3;      // J1 Pin 5 -> TB2 Pin 1 (ADC1_CH3)
const PIN_SOIL_POWER: i32 = 21;     // J2 Pin 8 -> TB2 Pin 2
const PIN_ULTRASONIC_TRIG: i32 = 7; // J2 Pin 3 -> TB1 Pin 2
const PIN_ULTRASONIC_ECHO: i32 = 20;// J2 Pin 7 -> TB1 Pin 3 (via R3/R4)
const _PIN_SUPPLY_ADC: i32 = 4;     // J1 Pin 4 -> R1/R2 divider (ADC1_CH4)
const PIN_STATUS_LED: i32 = 8;      // Onboard Blue LED (Active LOW)

// Default soil probe calibration (mV)
const DEFAULT_DRY_MV: i32 = 2600;
const DEFAULT_WET_MV: i32 = 1300;

fn read_adc1_mv(channel: u32) -> i32 {
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

fn force_ultrasonic_reset() {
    unsafe {
        gpio_set_direction(PIN_ULTRASONIC_ECHO, gpio_mode_t_GPIO_MODE_OUTPUT);
        gpio_set_level(PIN_ULTRASONIC_ECHO, 0);
        esp_rom_delay_us(10);
        gpio_set_direction(PIN_ULTRASONIC_ECHO, gpio_mode_t_GPIO_MODE_INPUT);
    }
    sleep(Duration::from_millis(30));
}

fn read_ultrasonic_distance_cm() -> Option<f32> {
    let mut samples = [0.0f32; 5];
    let mut valid_samples = 0;

    for _ in 0..5 {
        unsafe {
            // If ECHO is stuck HIGH before trigger, clear latch immediately
            if gpio_get_level(PIN_ULTRASONIC_ECHO) == 1 {
                force_ultrasonic_reset();
            }

            // Send 10µs TRIG pulse
            gpio_set_level(PIN_ULTRASONIC_TRIG, 0);
            esp_rom_delay_us(2);
            gpio_set_level(PIN_ULTRASONIC_TRIG, 1);
            esp_rom_delay_us(10);
            gpio_set_level(PIN_ULTRASONIC_TRIG, 0);

            // Wait for ECHO pin to go HIGH (up to 10ms wait)
            let t_trig = esp_timer_get_time();
            let mut started = false;
            while (esp_timer_get_time() - t_trig) < 10_000 {
                if gpio_get_level(PIN_ULTRASONIC_ECHO) == 1 {
                    started = true;
                    break;
                }
            }

            if !started {
                sleep(Duration::from_millis(10));
                continue;
            }

            // Measure ECHO HIGH pulse duration with a 35ms timeout
            let t_start = esp_timer_get_time();
            let mut ended = false;
            while (esp_timer_get_time() - t_start) < 35_000 {
                if gpio_get_level(PIN_ULTRASONIC_ECHO) == 0 {
                    ended = true;
                    break;
                }
            }
            let t_end = esp_timer_get_time();

            if !ended {
                force_ultrasonic_reset();
                continue;
            }

            let duration_us = t_end - t_start;
            let distance = echo_us_to_cm(duration_us);
            if is_valid_distance(distance) {
                samples[valid_samples] = distance;
                valid_samples += 1;
            }
        }

        // 60ms acoustic decay pause between pings
        sleep(Duration::from_millis(60));
    }

    if valid_samples == 0 {
        None
    } else {
        median_f32(&mut samples[..valid_samples])
    }
}

fn main() -> anyhow::Result<()> {
    link_patches();
    EspLogger::initialize_default();

    println!("\n========================================================");
    println!(" 🌿 Plant ESP32-C3 Live Sensor Verification Utility");
    println!("========================================================");
    println!(" - Canonical Pinout: https://lastminuteengineers.com/esp32-c3-super-mini-pinout-reference/");
    println!(" - Soil Sensor:      DATA = GPIO 3 (TB2-1), POWER = GPIO 21 (TB2-2)");
    println!(" - Ultrasonic:       TRIG = GPIO 7 (TB1-2), ECHO  = GPIO 20 (TB1-3)");
    println!(" - 5V Supply Rail:   GPIO 4 (ADC1_CH4 via R1/R2 divider)");
    println!(" - Status LED:       GPIO 8 (Flashes on each sample cycle)");
    println!("--------------------------------------------------------");
    println!(" 💡 How to test:");
    println!("   1. Touch the soil probe or dip it in a cup of water:");
    println!("      -> Soil mV will drop, Moisture % will rise.");
    println!("   2. Wave your hand 10-30cm in front of the ultrasonic sensor:");
    println!("      -> Distance cm will track your hand movement.");
    println!("========================================================\n");

    unsafe {
        // Configure ADC1 (12-bit, 11dB attenuation for full 0–3.1V scale)
        adc1_config_width(adc_bits_width_t_ADC_WIDTH_BIT_12);
        adc1_config_channel_atten(adc1_channel_t_ADC1_CHANNEL_3, adc_atten_t_ADC_ATTEN_DB_11);
        adc1_config_channel_atten(adc1_channel_t_ADC1_CHANNEL_4, adc_atten_t_ADC_ATTEN_DB_11);

        // Configure Soil Power Switch (GPIO 21)
        gpio_reset_pin(PIN_SOIL_POWER);
        gpio_set_direction(PIN_SOIL_POWER, gpio_mode_t_GPIO_MODE_OUTPUT);
        gpio_set_level(PIN_SOIL_POWER, 0);

        // Configure Ultrasonic TRIG (GPIO 7)
        gpio_reset_pin(PIN_ULTRASONIC_TRIG);
        gpio_set_direction(PIN_ULTRASONIC_TRIG, gpio_mode_t_GPIO_MODE_OUTPUT);
        gpio_set_level(PIN_ULTRASONIC_TRIG, 0);

        // Configure Ultrasonic ECHO (GPIO 20)
        gpio_reset_pin(PIN_ULTRASONIC_ECHO);
        gpio_set_direction(PIN_ULTRASONIC_ECHO, gpio_mode_t_GPIO_MODE_INPUT);

        // Configure Status LED (GPIO 8)
        gpio_reset_pin(PIN_STATUS_LED);
        gpio_set_direction(PIN_STATUS_LED, gpio_mode_t_GPIO_MODE_OUTPUT);
        gpio_set_level(PIN_STATUS_LED, 1); // Active LOW: 1 = OFF
    }

    let mut cycle: u32 = 0;

    loop {
        cycle += 1;

        // Turn on LED during sampling
        unsafe { gpio_set_level(PIN_STATUS_LED, 0); }

        // --- 1. Read Soil Moisture (Switched Power on GPIO 21) ---
        unsafe {
            gpio_set_level(PIN_SOIL_POWER, 1);
        }
        sleep(Duration::from_millis(35)); // Allow oscillator on probe to stabilize

        let soil_mv = read_adc1_mv(adc1_channel_t_ADC1_CHANNEL_3);
        let raw_soil_bits = unsafe { adc1_get_raw(adc1_channel_t_ADC1_CHANNEL_3) };

        unsafe {
            gpio_set_level(PIN_SOIL_POWER, 0); // Power off immediately to prevent electrolysis
        }

        let moisture_pct = soil_percent(soil_mv, DEFAULT_DRY_MV, DEFAULT_WET_MV);

        // --- 2. Read Ultrasonic Distance ---
        let distance_cm = read_ultrasonic_distance_cm();

        // --- 3. Read 5V Rail Monitor ---
        let v5_mv = read_adc1_mv(adc1_channel_t_ADC1_CHANNEL_4);
        let v5_volts = (v5_mv as f32 * 2.0) / 1000.0;

        // Turn off LED
        unsafe { gpio_set_level(PIN_STATUS_LED, 1); }

        // Print Telemetry Summary
        let dist_str = match distance_cm {
            Some(d) => format!("{:5.1} cm", d),
            None => "NO ECHO / OUT OF RANGE".to_string(),
        };

        println!(
            "[{:04}] 5V Rail: {:.2}V | Soil: {:4} mV ({:4} bits, {:5.1}%) | Ultrasonic: {}",
            cycle, v5_volts, soil_mv, raw_soil_bits, moisture_pct, dist_str
        );

        sleep(Duration::from_millis(965));
    }
}
