//! Hardware Diagnostic: Toggles TB2 Pin 2 (Soil Sensor Switched VCC on GPIO 21) every 2 seconds
//! to allow easy voltmeter verification on TB2 Pin 2.
//!
//! Canonical Pinout Reference:
//! https://lastminuteengineers.com/esp32-c3-super-mini-pinout-reference/
//!
//! Synchronizes the onboard Blue LED (GPIO 8) with GPIO 21:
//! - When GPIO 21 is 3.3V (ON)  -> LED is ON  (voltmeter on TB2 Pin 2 reads ~3.3V)
//! - When GPIO 21 is 0.0V (OFF) -> LED is OFF (voltmeter on TB2 Pin 2 reads 0.0V)

use std::thread::sleep;
use std::time::Duration;

use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::{
    adc1_channel_t_ADC1_CHANNEL_3, adc1_config_channel_atten, adc1_config_width,
    adc1_get_raw, adc_atten_t_ADC_ATTEN_DB_11, adc_bits_width_t_ADC_WIDTH_BIT_12,
    adc_unit_t_ADC_UNIT_1, esp_adc_cal_characteristics_t, esp_adc_cal_characterize,
    esp_adc_cal_raw_to_voltage, gpio_mode_t_GPIO_MODE_OUTPUT, gpio_reset_pin, gpio_set_direction,
    gpio_set_level, link_patches,
};

const PIN_SOIL_POWER: i32 = 21; // Physically connected to TB2 Pin 2 (J2 Pin 8)
const PIN_STATUS_LED: i32 = 8;  // Active low on SuperMini

fn read_soil_adc_mv() -> i32 {
    unsafe {
        let raw = adc1_get_raw(adc1_channel_t_ADC1_CHANNEL_3);
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

fn main() -> anyhow::Result<()> {
    link_patches();
    EspLogger::initialize_default();

    println!("\n========================================================");
    println!(" ⚡ ESP32-C3 TB2 Pin 2 (GPIO 21) Voltmeter Toggle Diagnostic");
    println!("========================================================");
    println!(" - Reference:        https://lastminuteengineers.com/esp32-c3-super-mini-pinout-reference/");
    println!(" - Measuring Target: TB2 Pin 2 (Center pin) vs TB2 Pin 3 (GND)");
    println!(" - Controlling Pin:  GPIO 21 (Switched VCC to TB2 Pin 2)");
    println!(" - Data Sensor Pin:  GPIO 3  (TB2 Pin 1 AOUT, ADC1_CH3)");
    println!(" - Cycle:            2s HIGH (3.3V) <---> 2s LOW (0.0V)");
    println!(" - Blue LED:         ON when 3.3V, OFF when 0.0V");
    println!(" Press Ctrl+C in console to stop.");
    println!("--------------------------------------------------------\n");

    unsafe {
        // Configure ADC1 on GPIO 3 to monitor DATA simultaneously
        adc1_config_width(adc_bits_width_t_ADC_WIDTH_BIT_12);
        adc1_config_channel_atten(adc1_channel_t_ADC1_CHANNEL_3, adc_atten_t_ADC_ATTEN_DB_11);

        // Configure GPIO 21 as direct Push-Pull Output
        gpio_reset_pin(PIN_SOIL_POWER);
        gpio_set_direction(PIN_SOIL_POWER, gpio_mode_t_GPIO_MODE_OUTPUT);
        gpio_set_level(PIN_SOIL_POWER, 0);

        // Also configure GPIO 5 as output and toggle together for completeness
        gpio_reset_pin(5);
        gpio_set_direction(5, gpio_mode_t_GPIO_MODE_OUTPUT);
        gpio_set_level(5, 0);

        // Configure onboard Blue LED
        gpio_reset_pin(PIN_STATUS_LED);
        gpio_set_direction(PIN_STATUS_LED, gpio_mode_t_GPIO_MODE_OUTPUT);
        gpio_set_level(PIN_STATUS_LED, 1); // Start OFF (active low)
    }

    let mut state = false;
    let mut count: u32 = 0;

    loop {
        count += 1;
        state = !state;

        unsafe {
            if state {
                // DRIVE HIGH (3.3V)
                gpio_set_level(PIN_SOIL_POWER, 1);
                gpio_set_level(5, 1);
                gpio_set_level(PIN_STATUS_LED, 0); // Blue LED ON
            } else {
                // DRIVE LOW (0.0V)
                gpio_set_level(PIN_SOIL_POWER, 0);
                gpio_set_level(5, 0);
                gpio_set_level(PIN_STATUS_LED, 1); // Blue LED OFF
            }
        }

        // Wait 500ms into the phase before reading ADC so voltage settles
        sleep(Duration::from_millis(500));

        let data_mv = read_soil_adc_mv();

        if state {
            println!(
                "[{:03}] 🟢 STATE: HIGH (3.3V)  | LED: ON  | TB2 Pin 2: ~3.3V | TB2 Pin 1 (GPIO 3 DATA): {} mV",
                count, data_mv
            );
        } else {
            println!(
                "[{:03}] 🔴 STATE: LOW  (0.0V)  | LED: OFF | TB2 Pin 2: 0.0V  | TB2 Pin 1 (GPIO 3 DATA): {} mV",
                count, data_mv
            );
        }

        // Wait remaining 1500ms (total 2000ms per phase)
        sleep(Duration::from_millis(1500));
    }
}
