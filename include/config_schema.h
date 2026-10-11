#pragma once

#include <Arduino.h>
#include <vector>

/**
 * @file config_schema.h
 * @brief Strongly typed C++ structures for Plant Monitor configuration.
 */

struct SoilMoistureGaugeConfig {
    int trigger_threshold = 40;  // Percentage (%) below which watering triggers
    int dry_adc = 2800;          // ADC value in air (0% moisture)
    int wet_adc = 1100;          // ADC value in water (100% moisture)
};

struct WaterLevelGaugeConfig {
    float tank_height_cm = 30.0f;     // Total tank depth from sensor to bottom
    float sensor_offset_cm = 3.0f;    // Distance from sensor to 100% full water surface
    float min_safe_level_cm = 2.0f;   // Minimum water level before dry-run protection engages
};

struct VoltageGaugeConfig {
    float min_warning = 4.5f;   // 5V rail brownout / weak USB warning threshold
    float max_warning = 5.5f;   // 5V rail over-voltage warning
};

struct GaugesConfig {
    SoilMoistureGaugeConfig soil;
    WaterLevelGaugeConfig water;
    VoltageGaugeConfig voltage;
};

struct TimingConfig {
    int startup_delay_sec = 30;
    int check_interval_sec = 3600;
    int sensor_read_interval_sec = 600;
    int automated_water_duration_sec = 20;
    int manual_water_duration_sec = 10;
};

struct SafetyConfig {
    int max_continuous_pump_sec = 60;
    int min_time_between_cycles_sec = 300;
    bool daily_limit_enabled = true;
    int max_seconds_per_day = 60;
    bool night_protection_enabled = true;
    int night_start_hour = 22;  // 10 PM
    int night_end_hour = 9;     // 9 AM
};

struct SystemConfig {
    String hostname = "plants";
    String timezone = "Asia/Singapore";
    int gmt_offset = 8;
    std::vector<String> ntp_servers = {"time.google.com", "pool.ntp.org", "time.nist.gov"};
    int monitoring_interval_sec = 60;
};

struct AppConfig {
    SystemConfig system;
    GaugesConfig gauges;
    TimingConfig timing;
    SafetyConfig safety;
};
