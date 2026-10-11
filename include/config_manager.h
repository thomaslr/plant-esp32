#pragma once

#include <Arduino.h>
#include <LittleFS.h>
#include <ArduinoJson.h>
#include "config_schema.h"

/**
 * @file config_manager.h
 * @brief Helper for loading and saving AppConfig to LittleFS /config.json
 */

class ConfigManager {
public:
    static bool loadConfig(AppConfig& config, const char* filepath = "/config.json") {
        if (!LittleFS.begin(true)) {
            Serial.println("[FS] Failed to mount LittleFS");
            return false;
        }

        if (!LittleFS.exists(filepath)) {
            Serial.println("[FS] config.json does not exist. Using defaults.");
            saveConfig(config, filepath);
            return false;
        }

        File file = LittleFS.open(filepath, "r");
        if (!file) {
            Serial.println("[FS] Failed to open config.json for reading");
            return false;
        }

        JsonDocument doc;
        DeserializationError err = deserializeJson(doc, file);
        file.close();

        if (err) {
            Serial.printf("[FS] Failed to parse config.json: %s\n", err.c_str());
            return false;
        }

        // Parse system
        if (doc["system"]["hostname"]) config.system.hostname = doc["system"]["hostname"].as<String>();
        if (doc["system"]["timezone"]) config.system.timezone = doc["system"]["timezone"].as<String>();
        if (doc["system"]["gmt_offset"]) config.system.gmt_offset = doc["system"]["gmt_offset"].as<int>();

        // Parse gauges
        if (doc["gauges"]["soil_moisture"]["trigger_threshold"])
            config.gauges.soil.trigger_threshold = doc["gauges"]["soil_moisture"]["trigger_threshold"].as<int>();
        if (doc["gauges"]["soil_moisture"]["dry_adc"])
            config.gauges.soil.dry_adc = doc["gauges"]["soil_moisture"]["dry_adc"].as<int>();
        if (doc["gauges"]["soil_moisture"]["wet_adc"])
            config.gauges.soil.wet_adc = doc["gauges"]["soil_moisture"]["wet_adc"].as<int>();

        if (doc["gauges"]["water_level"]["tank_height_cm"])
            config.gauges.water.tank_height_cm = doc["gauges"]["water_level"]["tank_height_cm"].as<float>();
        if (doc["gauges"]["water_level"]["sensor_offset_cm"])
            config.gauges.water.sensor_offset_cm = doc["gauges"]["water_level"]["sensor_offset_cm"].as<float>();
        if (doc["gauges"]["water_level"]["min_safe_level_cm"])
            config.gauges.water.min_safe_level_cm = doc["gauges"]["water_level"]["min_safe_level_cm"].as<float>();

        if (doc["gauges"]["voltage"]["min_warning"])
            config.gauges.voltage.min_warning = doc["gauges"]["voltage"]["min_warning"].as<float>();
        if (doc["gauges"]["voltage"]["max_warning"])
            config.gauges.voltage.max_warning = doc["gauges"]["voltage"]["max_warning"].as<float>();

        // Parse timing
        if (doc["timing"]["startup_delay_sec"])
            config.timing.startup_delay_sec = doc["timing"]["startup_delay_sec"].as<int>();
        if (doc["timing"]["check_interval_sec"])
            config.timing.check_interval_sec = doc["timing"]["check_interval_sec"].as<int>();
        if (doc["timing"]["sensor_read_interval_sec"])
            config.timing.sensor_read_interval_sec = doc["timing"]["sensor_read_interval_sec"].as<int>();
        if (doc["timing"]["automated_water_duration_sec"])
            config.timing.automated_water_duration_sec = doc["timing"]["automated_water_duration_sec"].as<int>();
        if (doc["timing"]["manual_water_duration_sec"])
            config.timing.manual_water_duration_sec = doc["timing"]["manual_water_duration_sec"].as<int>();

        // Parse safety
        if (doc["safety"]["max_continuous_pump_sec"])
            config.safety.max_continuous_pump_sec = doc["safety"]["max_continuous_pump_sec"].as<int>();
        if (doc["safety"]["min_time_between_cycles_sec"])
            config.safety.min_time_between_cycles_sec = doc["safety"]["min_time_between_cycles_sec"].as<int>();
        if (doc["safety"]["daily_watering_limit"]["enabled"])
            config.safety.daily_limit_enabled = doc["safety"]["daily_watering_limit"]["enabled"].as<bool>();
        if (doc["safety"]["daily_watering_limit"]["max_seconds_per_day"])
            config.safety.max_seconds_per_day = doc["safety"]["daily_watering_limit"]["max_seconds_per_day"].as<int>();
        if (doc["safety"]["night_watering_protection"]["enabled"])
            config.safety.night_protection_enabled = doc["safety"]["night_watering_protection"]["enabled"].as<bool>();
        if (doc["safety"]["night_watering_protection"]["start_hour"])
            config.safety.night_start_hour = doc["safety"]["night_watering_protection"]["start_hour"].as<int>();
        if (doc["safety"]["night_watering_protection"]["end_hour"])
            config.safety.night_end_hour = doc["safety"]["night_watering_protection"]["end_hour"].as<int>();

        Serial.println("[FS] Configuration successfully loaded from LittleFS");
        return true;
    }

    static bool saveConfig(const AppConfig& config, const char* filepath = "/config.json") {
        JsonDocument doc;

        doc["system"]["hostname"] = config.system.hostname;
        doc["system"]["timezone"] = config.system.timezone;
        doc["system"]["gmt_offset"] = config.system.gmt_offset;

        JsonArray ntp = doc["system"]["ntp_servers"].to<JsonArray>();
        for (const auto& s : config.system.ntp_servers) {
            ntp.add(s);
        }

        doc["gauges"]["soil_moisture"]["trigger_threshold"] = config.gauges.soil.trigger_threshold;
        doc["gauges"]["soil_moisture"]["dry_adc"] = config.gauges.soil.dry_adc;
        doc["gauges"]["soil_moisture"]["wet_adc"] = config.gauges.soil.wet_adc;

        doc["gauges"]["water_level"]["tank_height_cm"] = config.gauges.water.tank_height_cm;
        doc["gauges"]["water_level"]["sensor_offset_cm"] = config.gauges.water.sensor_offset_cm;
        doc["gauges"]["water_level"]["min_safe_level_cm"] = config.gauges.water.min_safe_level_cm;

        doc["gauges"]["voltage"]["min_warning"] = config.gauges.voltage.min_warning;
        doc["gauges"]["voltage"]["max_warning"] = config.gauges.voltage.max_warning;

        doc["timing"]["startup_delay_sec"] = config.timing.startup_delay_sec;
        doc["timing"]["check_interval_sec"] = config.timing.check_interval_sec;
        doc["timing"]["sensor_read_interval_sec"] = config.timing.sensor_read_interval_sec;
        doc["timing"]["automated_water_duration_sec"] = config.timing.automated_water_duration_sec;
        doc["timing"]["manual_water_duration_sec"] = config.timing.manual_water_duration_sec;

        doc["safety"]["max_continuous_pump_sec"] = config.safety.max_continuous_pump_sec;
        doc["safety"]["min_time_between_cycles_sec"] = config.safety.min_time_between_cycles_sec;
        doc["safety"]["daily_watering_limit"]["enabled"] = config.safety.daily_limit_enabled;
        doc["safety"]["daily_watering_limit"]["max_seconds_per_day"] = config.safety.max_seconds_per_day;
        doc["safety"]["night_watering_protection"]["enabled"] = config.safety.night_protection_enabled;
        doc["safety"]["night_watering_protection"]["start_hour"] = config.safety.night_start_hour;
        doc["safety"]["night_watering_protection"]["end_hour"] = config.safety.night_end_hour;

        File file = LittleFS.open(filepath, "w");
        if (!file) {
            Serial.println("[FS] Failed to open config.json for writing");
            return false;
        }

        serializeJsonPretty(doc, file);
        file.close();
        Serial.println("[FS] Configuration saved to LittleFS");
        return true;
    }
};
