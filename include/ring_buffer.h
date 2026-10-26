#pragma once

#include <Arduino.h>
#include <ArduinoJson.h>
#include <vector>

/**
 * @file ring_buffer.h
 * @brief Thread-safe in-memory circular ring buffer for sensor logs and diagnostic events.
 * Eliminates SPI flash wear and MicroPython heap fragmentation.
 */

struct SensorReading {
    uint32_t timestamp = 0;        // Unix epoch or uptime in seconds
    float soil_percent = 0.0f;     // Calculated moisture percentage
    int soil_raw_adc = 0;          // Calibrated ADC millivolts / raw counts
    float water_level_cm = 0.0f;   // Calculated water depth in tank
    float water_percent = 0.0f;    // Tank fullness percentage
    float distance_cm = 0.0f;      // Raw ultrasonic distance to water surface
    float supply_voltage = 0.0f;   // 5V rail measured voltage
    float battery_voltage = 0.0f;  // 18650 Battery measured voltage
    float chip_temp = 0.0f;        // Internal ESP32-C3 silicon temperature
    bool pump_active = false;      // True if pump was running during sample
};

template <size_t Capacity>
class SensorRingBuffer {
private:
    SensorReading buffer_[Capacity];
    size_t head_ = 0;
    size_t count_ = 0;
    portMUX_TYPE mux_ = portMUX_INITIALIZER_UNLOCKED;

public:
    SensorRingBuffer() = default;

    void push(const SensorReading& reading) {
        portENTER_CRITICAL(&mux_);
        buffer_[head_] = reading;
        head_ = (head_ + 1) % Capacity;
        if (count_ < Capacity) {
            count_++;
        }
        portEXIT_CRITICAL(&mux_);
    }

    size_t size() const {
        return count_;
    }

    // Fetches up to 'n' most recent readings (newest first)
    std::vector<SensorReading> getRecent(size_t n) {
        std::vector<SensorReading> result;
        portENTER_CRITICAL(&mux_);
        size_t available = count_;
        if (n > available) n = available;
        result.reserve(n);

        for (size_t i = 0; i < n; ++i) {
            // Calculate reverse index from newest to oldest
            size_t idx = (head_ + Capacity - 1 - i) % Capacity;
            result.push_back(buffer_[idx]);
        }
        portEXIT_CRITICAL(&mux_);
        return result;
    }

    // Serializes recent readings into an ArduinoJson document
    void serializeToJson(JsonArray& array, size_t n) {
        std::vector<SensorReading> readings = getRecent(n);
        for (const auto& r : readings) {
            JsonObject obj = array.add<JsonObject>();
            obj["timestamp"] = r.timestamp;
            obj["soil_percent"] = roundf(r.soil_percent * 10.0f) / 10.0f;
            obj["soil_raw"] = r.soil_raw_adc;
            obj["water_level_cm"] = roundf(r.water_level_cm * 10.0f) / 10.0f;
            obj["water_percent"] = roundf(r.water_percent * 10.0f) / 10.0f;
            obj["distance_cm"] = roundf(r.distance_cm * 10.0f) / 10.0f;
            obj["supply_voltage"] = roundf(r.supply_voltage * 100.0f) / 100.0f;
            obj["battery_voltage"] = roundf(r.battery_voltage * 100.0f) / 100.0f;
            obj["chip_temp"] = roundf(r.chip_temp * 10.0f) / 10.0f;
            obj["pump_active"] = r.pump_active;
        }
    }
};
