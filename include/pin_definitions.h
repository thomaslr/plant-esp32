#pragma once

#include <Arduino.h>

/**
 * @file pin_definitions.h
 * @brief Pin definitions for the Plant Monitor on ESP32-C3 SuperMini.
 * 
 * Hardware Layout Reference:
 * - Breadboard: 15-column mini breadboard
 * - ESP32-C3 SuperMini straddles rows F (Top) and B (Bottom) across Cols 1-8
 * - Terminal Blocks: TB1 (Row J top, odd cols 9, 11, 13, 15), TB2 (Row F top, even cols 10, 12, 14), TB3 (Row A bottom, cols 13, 15)
 */

// =============================================================================
// Analog Sensors (ADC1)
// =============================================================================
// GPIO 0 (ADC1_CH0): 5V Supply Rail Monitor via 10k/10k divider + 100nF cap (Col 4 Top)
#define PIN_SUPPLY_MONITOR_ADC    0

// GPIO 1 (ADC1_CH1): Capacitive Soil Moisture Sensor AOUT (Col 5 Top, from TB2 Pin 2)
#define PIN_SOIL_MOISTURE_ADC     1

// GPIO 2: Switched VCC Power for Capacitive Soil Sensor (Col 6 Top -> TB2 Pin 1 VCC)
// Duty-cycled: HIGH only during ADC sampling to eliminate 7mA idle draw and galvanic corrosion
#define PIN_SOIL_POWER            2

// =============================================================================
// Ultrasonic Sensor (HC-SR04)
// =============================================================================
// GPIO 3: HC-SR04 TRIG (Col 7 Top -> TB1 Pin 2 TRIG)
#define PIN_ULTRASONIC_TRIG       3

// GPIO 4: HC-SR04 ECHO (Col 8 Top -> 1k/2k Divider -> TB1 Pin 3 ECHO)
#define PIN_ULTRASONIC_ECHO       4

// =============================================================================
// Actuators & Indicators
// =============================================================================
// GPIO 7: Pump MOSFET Gate via 1k resistor & 100k pull-down (Col 6 Bottom / Row A -> Q1 FQP30N06L Gate in Col 12)
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
