//! Raw Hardware Pin & Voltmeter Exerciser
//!
//! Canonical Pinout Reference:
//! https://lastminuteengineers.com/esp32-c3-super-mini-pinout-reference/
//!
//! Completely standalone: no Wi-Fi, no sensors, no timeouts, no supervisors.
//! Drives GPIO 21 hard HIGH (3.3V) for 3s, then hard LOW (0V) for 3s.
//! Synchronizes with the onboard Blue LED (GPIO 8).

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

const PIN_SOIL_POWER: i32 = 21; // TB2 Pin 2 (J2 Pin 8)
const PIN_STATUS_LED: i32 = 8;  // SuperMini onboard LED (Active LOW)

fn read_gpio3_mv() -> i32 {
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
    println!(" 🛠️ RAW HARDWARE PIN VOLTMETER TEST (Zero Dependencies)");
    println!("========================================================");
    println!(" Reference: https://lastminuteengineers.com/esp32-c3-super-mini-pinout-reference/");
    println!(" Cycle:     3 seconds HIGH (3.3V) <---> 3 seconds LOW (0.0V)");
    println!(" Target:    TB2 Pin 2 (Switched VCC / GPIO 21) relative to TB2 Pin 3 (GND)");
    println!(" Onboard Blue LED (GPIO 8) will turn ON when HIGH, OFF when LOW.");
    println!(" Press Ctrl+C in console to stop.");
    println!("--------------------------------------------------------\n");

    unsafe {
        // Setup GPIO 3 (ADC1_CH3) for TB2 Pin 1 (Soil DATA)
        adc1_config_width(adc_bits_width_t_ADC_WIDTH_BIT_12);
        adc1_config_channel_atten(adc1_channel_t_ADC1_CHANNEL_3, adc_atten_t_ADC_ATTEN_DB_11);

        // Setup GPIO 21 as direct Push-Pull Output
        gpio_reset_pin(PIN_SOIL_POWER);
        gpio_set_direction(PIN_SOIL_POWER, gpio_mode_t_GPIO_MODE_OUTPUT);
        gpio_set_level(PIN_SOIL_POWER, 0);

        // Setup GPIO 8 (Blue LED)
        gpio_reset_pin(PIN_STATUS_LED);
        gpio_set_direction(PIN_STATUS_LED, gpio_mode_t_GPIO_MODE_OUTPUT);
        gpio_set_level(PIN_STATUS_LED, 1); // OFF
    }

    let mut cycle: u32 = 0;

    loop {
        cycle += 1;

        // PHASE 1: HIGH (3.3V)
        println!("\n>>> [{:03}] DRIVING HIGH (3.3V)... (LED: ON)", cycle);
        unsafe {
            gpio_set_level(PIN_SOIL_POWER, 1);
            gpio_set_level(PIN_STATUS_LED, 0); // ON (active LOW)
        }

        for sec in 1..=3 {
            sleep(Duration::from_millis(1000));
            let mv = read_gpio3_mv();
            println!("    HIGH +{}s: Voltmeter on TB2 Pin 2 should read ~3.3V | TB2 Pin 1 (GPIO 3 DATA): {} mV", sec, mv);
        }

        // PHASE 2: LOW (0.0V)
        println!("\n>>> [{:03}] DRIVING LOW (0.0V)... (LED: OFF)", cycle);
        unsafe {
            gpio_set_level(PIN_SOIL_POWER, 0);
            gpio_set_level(PIN_STATUS_LED, 1); // OFF
        }

        for sec in 1..=3 {
            sleep(Duration::from_millis(1000));
            let mv = read_gpio3_mv();
            println!("    LOW  +{}s: Voltmeter on TB2 Pin 2 should read 0.0V  | TB2 Pin 1 (GPIO 3 DATA): {} mV", sec, mv);
        }
    }
}
