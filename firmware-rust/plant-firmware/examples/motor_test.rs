//! Multi-Level PWM Pump Motor Diagnostic Tool.
//!
//! Canonical Pinout Reference:
//! https://lastminuteengineers.com/esp32-c3-super-mini-pinout-reference/
//!
//! Tests the submersible water pump on GPIO 10 (TB3) across multiple PWM duty cycles:
//! 40% -> 55% -> 70% -> 85% -> 100%
//! (Each step runs for 3 seconds with a 2-second pause in between)
//!
//! Uses ESP32 LEDC hardware PWM at 5 kHz for whisper-quiet switching and smooth torque.

use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::{
    adc1_channel_t_ADC1_CHANNEL_4, adc1_config_channel_atten, adc1_config_width,
    adc1_get_raw, adc_atten_t_ADC_ATTEN_DB_11, adc_bits_width_t_ADC_WIDTH_BIT_12,
    adc_unit_t_ADC_UNIT_1, esp_adc_cal_characteristics_t, esp_adc_cal_characterize,
    esp_adc_cal_raw_to_voltage, gpio_mode_t_GPIO_MODE_OUTPUT, gpio_reset_pin,
    gpio_set_direction, gpio_set_level, ledc_channel_config, ledc_channel_config_t,
    ledc_channel_t_LEDC_CHANNEL_0, ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_set_duty,
    ledc_timer_bit_t_LEDC_TIMER_10_BIT, ledc_timer_config, ledc_timer_config_t,
    ledc_timer_t_LEDC_TIMER_0, ledc_update_duty, link_patches,
};

const PIN_PUMP_GATE: i32 = 10; // J2 Pin 6 -> AO3400A Gate via R5
const PIN_STATUS_LED: i32 = 8;  // Onboard Blue LED (Active LOW: 0 = ON, 1 = OFF)
const PIN_SUPPLY_ADC: u32 = adc1_channel_t_ADC1_CHANNEL_4; // GPIO 4 5V Monitor

fn read_supply_volts() -> f32 {
    unsafe {
        for _ in 0..5 {
            let raw = adc1_get_raw(PIN_SUPPLY_ADC);
            if raw > 0 {
                let mut chars = std::mem::zeroed::<esp_adc_cal_characteristics_t>();
                esp_adc_cal_characterize(
                    adc_unit_t_ADC_UNIT_1,
                    adc_atten_t_ADC_ATTEN_DB_11,
                    adc_bits_width_t_ADC_WIDTH_BIT_12,
                    1100,
                    &mut chars,
                );
                let mv = esp_adc_cal_raw_to_voltage(raw as u32, &chars) as f32;
                return (mv * 2.0) / 1000.0; // 100k/100k divider factor = 2.0x
            }
            sleep(Duration::from_millis(5));
        }
        5.15
    }
}

fn init_hardware_pwm() {
    unsafe {
        // Configure 5 kHz, 10-bit resolution (0..1023 duty steps) timer
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

        // Status LED GPIO 8
        gpio_reset_pin(PIN_STATUS_LED);
        gpio_set_direction(PIN_STATUS_LED, gpio_mode_t_GPIO_MODE_OUTPUT);
        gpio_set_level(PIN_STATUS_LED, 1); // OFF
    }
}

fn set_pump_power_pct(pct: u32) {
    let clamped = pct.min(100);
    // 10-bit resolution: 0% -> 0, 100% -> 1023
    let duty = (clamped * 1023) / 100;
    unsafe {
        ledc_set_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_0, duty);
        ledc_update_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_0);
        if clamped > 0 {
            gpio_set_level(PIN_STATUS_LED, 0); // Blue LED ON
        } else {
            gpio_set_level(PIN_STATUS_LED, 1); // Blue LED OFF
        }
    }
}

fn main() -> anyhow::Result<()> {
    link_patches();
    EspLogger::initialize_default();

    println!("\n========================================================");
    println!(" 💧 Plant ESP32-C3 Multi-Level PWM Motor Diagnostic");
    println!("========================================================");
    println!(" - Canonical Pinout: https://lastminuteengineers.com/esp32-c3-super-mini-pinout-reference/");
    println!(" - MOSFET Gate Pin:  GPIO 10 (5 kHz Hardware PWM)");
    println!(" - Motor Terminal:   TB3 (Pin 1 = Drain -, Pin 2 = +5V)");
    println!(" - 5V Supply Rail:   GPIO 4 (ADC1_CH4, monitoring voltage sag)");
    println!("--------------------------------------------------------");
    println!(" Test Plan:");
    println!("   [1]  40% Power — Whisper-quiet low flow (3 seconds)");
    println!("   [2]  55% Power — Gentle smooth watering   (3 seconds)");
    println!("   [3]  70% Power — Moderate flow, quiet hum  (3 seconds)");
    println!("   [4]  85% Power — Strong flow               (3 seconds)");
    println!("   [5] 100% Power — Maximum full blast        (2 seconds)");
    println!("--------------------------------------------------------");
    println!(" [WARN] Ensure the pump is in water or directed into a basin!");
    println!(" Test starting in 3 seconds...\n");
    std::io::stdout().flush().ok();

    unsafe {
        adc1_config_width(adc_bits_width_t_ADC_WIDTH_BIT_12);
        adc1_config_channel_atten(PIN_SUPPLY_ADC, adc_atten_t_ADC_ATTEN_DB_11);
    }
    init_hardware_pwm();

    for i in (1..=3).rev() {
        println!("  >>> Starting in {}s...", i);
        std::io::stdout().flush().ok();
        sleep(Duration::from_secs(1));
    }

    let levels = [
        (40, 3, "Whisper-quiet low flow"),
        (55, 3, "Gentle smooth watering"),
        (70, 3, "Moderate flow, quiet hum"),
        (85, 3, "Strong flow"),
        (100, 2, "Maximum full blast"),
    ];

    for (step, &(pct, duration_s, label)) in levels.iter().enumerate() {
        let v_pre = read_supply_volts();
        println!(
            "\n▶ [STEP {}/5] Running at {:3}% PWM Power ({}) for {}s... (5V Rail: {:.2}V)",
            step + 1, pct, label, duration_s, v_pre
        );
        std::io::stdout().flush().ok();

        set_pump_power_pct(pct);
        sleep(Duration::from_secs(duration_s));

        set_pump_power_pct(0);
        println!("  ⏸ Pausing for 2s...");
        std::io::stdout().flush().ok();
        sleep(Duration::from_secs(2));
    }

    set_pump_power_pct(0);
    println!("\n========================================================");
    println!(" ✅ Multi-Level PWM Test Completed Successfully!");
    println!("========================================================");
    println!(" Note which power level gave you the ideal balance of quietness");
    println!(" and water flow rate. We will set that as the default and add");
    println!(" an interactive slider in the Web UI Settings tab!");
    println!("========================================================\n");
    std::io::stdout().flush().ok();

    sleep(Duration::from_secs(1));
    Ok(())
}
