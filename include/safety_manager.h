#pragma once

#include <Arduino.h>
#include "pin_definitions.h"
#include "config_schema.h"

/**
 * @file safety_manager.h
 * @brief Safety supervisor enforcing pump run limits, cooldowns, daily caps, and dry-run cutoffs.
 */

class SafetyManager {
public:
    explicit SafetyManager(const AppConfig& config);

    void begin();

    // Call from high-priority FreeRTOS safety task (checks continuous runtime and timeout)
    void tick();

    // Watering requests
    bool requestWatering(int duration_sec, const String& reason, float currentWaterLevelCm, int currentHour);
    void stopWatering(const String& reason);

    // Queries
    bool isPumpActive() const { return pump_active_; }
    uint32_t getActiveRuntimeSec() const;
    uint32_t getDailyRuntimeSec() const { return daily_watering_seconds_; }
    uint32_t getSecondsUntilNextAllowed() const;
    String getLastInhibitReason() const { return last_inhibit_reason_; }

    void resetDailyQuota();
    void updateConfig(const AppConfig& config);

private:
    AppConfig config_;
    bool pump_active_ = false;
    uint32_t pump_start_ms_ = 0;
    uint32_t target_duration_ms_ = 0;
    uint32_t last_pump_stop_ms_ = 0;
    uint32_t daily_watering_seconds_ = 0;
    String last_inhibit_reason_ = "None";

    void setPumpGate(bool state);
};
