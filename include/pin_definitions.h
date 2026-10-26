#pragma once

#include <Arduino.h>

/**
 * @file pin_definitions.h
 * @brief Pin definitions for the Plant Monitor on ESP32-C3 SuperMini.
 * 
 * Hardware Layout Reference:
 * - Top Header (J1):
 *   - Pin 1: 5V Power Bus
 *   - Pin 2: GND
 *   - Pin 3: +3.3V Regulated Output (Powers LDR divider, SHT30, BH1750)
 *   - Pin 4: GPIO 0 (ADC1_CH0) - Dedicated 5V Supply Rail Monitor (100k/100k divider R1/R2 + 100nF C1)
 *   - Pin 5: GPIO 1 (ADC1_CH1) - Capacitive Soil Moisture Sensor Analog Read
 *   - Pin 6: GPIO 2 - Unconnected / Spare (Strapping pin kept free to prevent boot failures)
 *   - Pin 7: GPIO 3 (ADC1_CH3) - Ambient Light LDR Voltage Divider (GL55xx photoresistor + 10k SMD R7)
 *   - Pin 8: GPIO 4 (ADC1_CH4) - Dedicated 18650 Battery Voltage Monitor (100k/100k divider R8/R9 + 100nF C6)
 * 
 * - External Power & Solar Terminal (TB4 - Rev 2.3):
 *   - Pin 1: EXT_5V (5.0V regulated input from solar charger / DD05CVSA boost module)
 *   - Pin 2: GND
 *   - Pin 3: VBAT_SENSE (Direct connection to 18650 3.7V Li-ion battery positive terminal)
 *   - Anti-Backfeeding: D2 (SS34 3A Schottky diode) isolates EXT_5V from USB port
 *   - Dedicated Dual ADC Channels: Concurrent real-time sampling in software without any manual jumpers!
 * 
 * - Bottom Header (J2):
 *   - Pin 1: GPIO 21 - I2C SCL (Clock for SHT30 & BH1750 Expansion Headers)
 *   - Pin 2: GPIO 20 - I2C SDA (Data for SHT30 & BH1750 Expansion Headers)
 *   - Pin 3: GPIO 10 - Ultrasonic HC-SR04 TRIG (Digital Output to TB1 Pin 2)
 *   - Pin 4: GPIO 9  - BOOT Button / Optional Manual Water Tactile Switch
 *   - Pin 5: GPIO 8  - Onboard System Status Blue LED (Active LOW)
 *   - Pin 6: GPIO 7  - Submersible Pump MOSFET Gate (Low-Side Switch)
 *   - Pin 7: GPIO 6  - Ultrasonic HC-SR04 ECHO (Digital Input via 1k/2k divider R3/R4 from TB1 Pin 3)
 *   - Pin 8: GPIO 5  - Switched VCC Power for Capacitive Soil Moisture Sensor
 */

// =============================================================================
// Analog Sensors (ADC1 — 100% Safe with Concurrent Active Wi-Fi)
// =============================================================================
// GPIO 0 (ADC1_CH0): Dedicated 5V Supply Rail Monitor via 100k/100k divider + 100nF filter capacitor
#define PIN_SUPPLY_MONITOR_ADC    0

// GPIO 4 (ADC1_CH4): Dedicated 18650 Battery Voltage Monitor via 100k/100k divider + 100nF filter capacitor
// Measures raw 18650 Li-ion battery voltage from TB4 Pin 3 (21µA negligible drain)
#define PIN_BATTERY_MONITOR_ADC   4

// GPIO 1 (ADC1_CH1): Capacitive Soil Moisture Sensor AOUT (from TB2 Pin 1)
#define PIN_SOIL_MOISTURE_ADC     1

// GPIO 3 (ADC1_CH3): Ambient Light Sensor LDR Divider (GL55xx photoresistor + 10k SMD R7)
// Non-strapping pin: 100% immune to dark/unpopulated startup boot hangs
#define PIN_LDR_LIGHT_ADC         3

// =============================================================================
// Switched Power (Digital Output)
// =============================================================================
// GPIO 5: Switched VCC Power for Capacitive Soil Sensor (TB2 Pin 2)
// Duty-cycled: HIGH only during ADC sampling to eliminate idle power and galvanic corrosion
#define PIN_SOIL_POWER            5

// =============================================================================
// Ultrasonic Sensor (HC-SR04)
// =============================================================================
// GPIO 10: HC-SR04 TRIG (TB1 Pin 2 TRIG) - Digital Output
#define PIN_ULTRASONIC_TRIG       10

// GPIO 6: HC-SR04 ECHO (TB1 Pin 3 ECHO via 1k/2k divider) - Digital Input
#define PIN_ULTRASONIC_ECHO       6

// =============================================================================
// I2C Expansion Bus (SHT30 Temp/Humidity & BH1750 Ambient Light)
// =============================================================================
// GPIO 20: I2C SDA (Data)
#define PIN_I2C_SDA               20

// GPIO 21: I2C SCL (Clock)
#define PIN_I2C_SCL               21

// =============================================================================
// Actuators & Indicators
// =============================================================================
// GPIO 7: Pump MOSFET Gate via 1k resistor & 100k pull-down (AO3400A / FQP30N06L Gate)
#define PIN_PUMP_GATE             7

// GPIO 8: Onboard Blue LED on ESP32-C3 SuperMini (Active LOW)
#define PIN_STATUS_LED            8
#define LED_ON                    LOW
#define LED_OFF                   HIGH

// =============================================================================
// Reserved Pins (DO NOT USE FOR SENSORS)
// =============================================================================
// GPIO 18: USB D- (Native USB-CDC / JTAG)
// GPIO 19: USB D+ (Native USB-CDC / JTAG)
