//! Standalone CLI Soil Moisture Sensor Hardware Diagnostic Tool.
//!
//! Canonical Pinout Reference:
//! https://lastminuteengineers.com/esp32-c3-super-mini-pinout-reference/
//!
//! Samples GPIO 3 (ADC1 Channel 3) connected to TB2 Pin 1 (AOUT) with duty-cycled
//! probe power on GPIO 21 connected to TB2 Pin 2 (Switched VCC) once every second,
//! displaying raw ADC bits, calibrated millivolts, and estimated moisture percentage in real-time.

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

const PIN_SOIL_POWER: i32 = 21; // TB2 Pin 2 (Switched VCC)
const PIN_STATUS_LED: i32 = 8;  // Active low on SuperMini

fn main() -> anyhow::Result<()> {
    link_patches();
    EspLogger::initialize_default();

    println!("\n==================================================");
    println!(" 🌱 Plant ESP32-C3 Soil Moisture Sensor Diagnostics");
    println!("==================================================");
    println!(" - Reference:  https://lastminuteengineers.com/esp32-c3-super-mini-pinout-reference/");
    println!(" - Sensor Pin: GPIO 3 (ADC1_CH3 / TB2 Pin 1 AOUT, 12-bit, 11dB)");
    println!(" - Power Pin:  GPIO 21 (Switched VCC / TB2 Pin 2, 50ms duty cycle)");
    println!(" - Status LED: GPIO 8 (Flashes on each sample)");
    println!(" - Sampling:   1 Hz (1 reading per second)");
    println!(" Press Ctrl+C to stop.");
    println!("--------------------------------------------------\n");

    unsafe {
        // Configure ADC1 12-bit width and 11dB attenuation on Channel 3 (GPIO 3)
        adc1_config_width(adc_bits_width_t_ADC_WIDTH_BIT_12);
        adc1_config_channel_atten(adc1_channel_t_ADC1_CHANNEL_3, adc_atten_t_ADC_ATTEN_DB_11);

        // Configure switched soil power pin (GPIO 21) as output, default LOW
        gpio_reset_pin(PIN_SOIL_POWER);
        gpio_set_direction(PIN_SOIL_POWER, gpio_mode_t_GPIO_MODE_OUTPUT);
        gpio_set_level(PIN_SOIL_POWER, 0);

        // Configure status LED (GPIO 8) as output
        gpio_reset_pin(PIN_STATUS_LED);
        gpio_set_direction(PIN_STATUS_LED, gpio_mode_t_GPIO_MODE_OUTPUT);
        gpio_set_level(PIN_STATUS_LED, 1); // OFF (active LOW)
    }

    // Default calibration values (from AppConfig defaults)
    let dry_adc = 2700; // Typical dry air reading (~2600-2900 mV)
    let wet_adc = 1400; // Typical water immersion reading (~1200-1500 mV)

    let mut sample_count: u32 = 0;

    loop {
        sample_count += 1;

        unsafe {
            // Flash LED ON briefly to indicate sample acquisition
            gpio_set_level(PIN_STATUS_LED, 0);

            // 1. Power ON probe (drive GPIO 21 HIGH)
            gpio_set_level(PIN_SOIL_POWER, 1);
        }

        // 2. Allow 50ms for 555 oscillator and probe power to stabilize
        sleep(Duration::from_millis(50));

        // 3. Take 5 samples with 5ms spacing and median filter
        let mut raw_bits = [0i32; 5];
        let mut raw_mv = [0i32; 5];

        for i in 0..5 {
            unsafe {
                let raw = adc1_get_raw(adc1_channel_t_ADC1_CHANNEL_3);
                raw_bits[i] = raw;

                let mut chars = std::mem::zeroed::<esp_adc_cal_characteristics_t>();
                esp_adc_cal_characterize(
                    adc_unit_t_ADC_UNIT_1,
                    adc_atten_t_ADC_ATTEN_DB_11,
                    adc_bits_width_t_ADC_WIDTH_BIT_12,
                    1100,
                    &mut chars,
                );
                raw_mv[i] = if raw >= 0 {
                    esp_adc_cal_raw_to_voltage(raw as u32, &chars) as i32
                } else {
                    0
                };
            }
            sleep(Duration::from_millis(5));
        }

        unsafe {
            // 4. Power OFF probe immediately to eliminate corrosion
            gpio_set_level(PIN_SOIL_POWER, 0);
            gpio_set_level(PIN_STATUS_LED, 1); // LED OFF
        }

        raw_bits.sort_unstable();
        raw_mv.sort_unstable();

        let median_bits = raw_bits[2];
        let median_voltage = raw_mv[2];

        // Moisture calculation (inverted: high voltage = dry, low voltage = wet)
        let percent = if dry_adc <= wet_adc {
            0.0
        } else {
            let clamped = median_voltage.clamp(wet_adc, dry_adc);
            ((dry_adc - clamped) as f32 / (dry_adc - wet_adc) as f32) * 100.0
        };

        let status = if median_voltage > 2500 {
            "🌵 DRY (Air)"
        } else if median_voltage > 1900 {
            "🌱 MOIST"
        } else if median_voltage > 1100 {
            "💧 WET (Water)"
        } else if median_voltage > 400 {
            "🌊 SATURATED"
        } else {
            "⚠️ UNPLUGGED / 0V"
        };

        println!(
            "[{:04}] Raw ADC: {:4} bits | Voltage: {:4} mV | Moisture: {:5.1}% | State: {}",
            sample_count, median_bits, median_voltage, percent, status
        );

        sleep(Duration::from_millis(945)); // ~1 second total loop period
    }
}
