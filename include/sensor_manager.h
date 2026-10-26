#pragma once

#include <Arduino.h>
#include "pin_definitions.h"
#include "config_schema.h"
#include "ring_buffer.h"

/**
 * @file sensor_manager.h
 * @brief Manages HC-SR04 ultrasonic echo capture, calibrated ADC reads, and internal silicon temperature.
 */

class SensorManager {
public:
    explicit SensorManager(const AppConfig& config);

    void begin();
    
    // Core acquisition routine (called periodically by FreeRTOS sensor task)
    SensorReading readAllSensors(bool pumpActiveState);

    // Individual sensor routines
    float readUltrasonicDistanceCm();
    int readSoilMoistureRaw();
    float readSoilMoisturePercent();
    float readSupplyVoltage();
    float readBatteryVoltage();
    float readChipTemperature();

    // Stuck-sensor active recovery
    void forceUltrasonicReset();
    uint32_t getPreventiveResetCount() const { return preventive_resets_count_; }

    void updateConfig(const AppConfig& config);

private:
    AppConfig config_;
    uint32_t preventive_resets_count_ = 0;

    // Helper functions
    float calculateMedian(float values[], size_t size);
    int calculateMedianInt(int values[], size_t size);
};
