#![allow(dead_code)]

//! Hardware Pin Definitions for Plant ESP32-C3 SuperMini.
//!
//! Mirrors `include/pin_definitions.h` from the C++ firmware.

/// GPIO 4 (ADC1_CH4): Physical pin next to 3.3V pin on ESP32-C3 SuperMini. Dedicated 5V Supply Rail Monitor via 10k/10k divider.
pub const PIN_SUPPLY_MONITOR_ADC: i32 = 4;

/// GPIO 1 (ADC1_CH1): Capacitive Soil Moisture Sensor AOUT.
pub const PIN_SOIL_MOISTURE_ADC: i32 = 1;

/// GPIO 3 (ADC1_CH3): Ambient Light Sensor LDR Divider (GL55xx photoresistor + 10k SMD R7).
pub const PIN_LDR_LIGHT_ADC: i32 = 3;

/// Dedicated 18650 Battery Voltage Monitor (-1 for unpopulated / USB-only builds).
pub const PIN_BATTERY_MONITOR_ADC: i32 = -1;

/// GPIO 5: Switched VCC Power for Capacitive Soil Moisture Sensor.
/// Duty-cycled: HIGH only during ADC sampling to eliminate idle draw and galvanic corrosion.
pub const PIN_SOIL_POWER: i32 = 5;

/// GPIO 6: HC-SR04 ECHO digital input (stepped down via 1k/2k voltage divider).
pub const PIN_ULTRASONIC_ECHO: i32 = 6;

/// GPIO 7: Pump MOSFET Gate (Low-Side Switch, AO3400A / FQP30N06L Gate).
pub const PIN_PUMP_GATE: i32 = 7;

/// GPIO 8: Onboard Blue LED on ESP32-C3 SuperMini (Active LOW).
pub const PIN_STATUS_LED: i32 = 8;

/// GPIO 10: HC-SR04 TRIG digital output (10µs pulse).
pub const PIN_ULTRASONIC_TRIG: i32 = 10;

/// GPIO 20: I2C SDA (Data for SHT30 & BH1750 expansion headers).
pub const PIN_I2C_SDA: i32 = 20;

/// GPIO 21: I2C SCL (Clock for SHT30 & BH1750 expansion headers).
pub const PIN_I2C_SCL: i32 = 21;
