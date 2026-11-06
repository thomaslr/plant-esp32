#![allow(dead_code)]

//! Hardware Pin Definitions for Plant ESP32-C3 SuperMini.
//!
//! Canonical Pinout Reference:
//! https://lastminuteengineers.com/esp32-c3-super-mini-pinout-reference/
//!
//! Pin mapping matching the physical ESP32-C3 SuperMini plugged into
//! headers J1 (5V, GND, 3V3, G4, G3, G2, G1, G0) and J2 (G5, G6, G7, G8, G9, G10, G20, G21):
//!
//! Right Column (J1, USB-C at top):
//! - Pin 1: 5V
//! - Pin 2: GND
//! - Pin 3: 3V3
//! - Pin 4: GPIO 4  (ADC1_CH4) -> 5V Supply Rail Monitor (R1/R2 100k/100k divider + C1 100nF)
//! - Pin 5: GPIO 3  (ADC1_CH3) -> Capacitive Soil Moisture Sensor Analog In (TB2 Pin 1 / AOUT)
//! - Pin 6: GPIO 2  (ADC1_CH2) -> Unconnected (Strapping safe, zero boot hangs)
//! - Pin 7: GPIO 1  (ADC1_CH1) -> Ambient Light LDR Sensor (LDR1 + R7 10k divider + C4 100nF)
//! - Pin 8: GPIO 0  (ADC1_CH0) -> 18650 Battery Voltage Monitor (R8/R9 100k/100k divider from TB4 Pin 3)
//!
//! Left Column (J2, USB-C at top):
//! - Pin 1: GPIO 5  (ADC2_CH0) -> I2C SCL (Clock for optional SHT30/BH1750 modules)
//! - Pin 2: GPIO 6  (SPI2_SCK) -> I2C SDA (Data for optional SHT30/BH1750 modules)
//! - Pin 3: GPIO 7  (SPI2_MOSI)-> Ultrasonic Sensor TRIG (TB1 Pin 2, 10µs pulse output)
//! - Pin 4: GPIO 8  (LED)      -> Onboard Blue Status LED (Active LOW)
//! - Pin 5: GPIO 9  (BOOT)     -> Onboard BOOT Button
//! - Pin 6: GPIO 10 (SPI2_CS)  -> Pump MOSFET Gate Driver (AO3400A Gate via R5 1k / R6 100k)
//! - Pin 7: GPIO 20 (U0_RXD)   -> Ultrasonic Sensor ECHO (TB1 Pin 3 via R3/R4 1k/2k 3.3V divider)
//! - Pin 8: GPIO 21 (U0_TXD)   -> Switched Soil Moisture Sensor VCC (TB2 Pin 2, duty-cycled)

/// GPIO 4 (ADC1_CH4): Physical pin next to 3.3V pin on ESP32-C3 SuperMini. Dedicated 5V Supply Rail Monitor via 100k/100k divider.
pub const PIN_SUPPLY_MONITOR_ADC: i32 = 4;

/// GPIO 3 (ADC1_CH3): Capacitive Soil Moisture Sensor AOUT (TB2 Pin 1).
pub const PIN_SOIL_MOISTURE_ADC: i32 = 3;

/// GPIO 1 (ADC1_CH1): Ambient Light Sensor LDR Divider (GL55xx photoresistor + 10k SMD R7).
pub const PIN_LDR_LIGHT_ADC: i32 = 1;

/// GPIO 0 (ADC1_CH0): Dedicated 18650 Battery Voltage Monitor via 100k/100k divider from TB4 Pin 3 (-1 if unpopulated).
pub const PIN_BATTERY_MONITOR_ADC: i32 = 0;

/// GPIO 21: Switched VCC Power for Capacitive Soil Moisture Sensor (TB2 Pin 2).
/// Duty-cycled: HIGH only during ADC sampling to eliminate idle draw and galvanic corrosion.
pub const PIN_SOIL_POWER: i32 = 21;

/// GPIO 20: HC-SR04 ECHO digital input (stepped down via 1k/2k voltage divider from TB1 Pin 3).
pub const PIN_ULTRASONIC_ECHO: i32 = 20;

/// GPIO 10: Pump MOSFET Gate (Low-Side Switch, AO3400A / FQP30N06L Gate via R5 1k).
pub const PIN_PUMP_GATE: i32 = 10;

/// GPIO 8: Onboard Blue LED on ESP32-C3 SuperMini (Active LOW).
pub const PIN_STATUS_LED: i32 = 8;

/// GPIO 7: HC-SR04 TRIG digital output (10µs pulse to TB1 Pin 2).
pub const PIN_ULTRASONIC_TRIG: i32 = 7;

/// GPIO 6: I2C SDA (Data for SHT30 & BH1750 expansion headers).
pub const PIN_I2C_SDA: i32 = 6;

/// GPIO 5: I2C SCL (Clock for SHT30 & BH1750 expansion headers).
pub const PIN_I2C_SCL: i32 = 5;
