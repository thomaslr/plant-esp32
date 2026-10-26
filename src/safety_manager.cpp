#include "safety_manager.h"

SafetyManager::SafetyManager(const AppConfig& config)
    : config_(config) {}

void SafetyManager::begin() {
    digitalWrite(PIN_PUMP_GATE, LOW);
    pinMode(PIN_PUMP_GATE, OUTPUT);

    digitalWrite(PIN_STATUS_LED, LED_OFF);
    pinMode(PIN_STATUS_LED, OUTPUT);

    pump_active_ = false;
    last_pump_stop_ms_ = 0;
    daily_watering_seconds_ = 0;
    last_inhibit_reason_ = "System initializing";
}

void SafetyManager::updateConfig(const AppConfig& config) {
    config_ = config;
}

void SafetyManager::setPumpGate(bool state) {
    digitalWrite(PIN_PUMP_GATE, state ? HIGH : LOW);
    digitalWrite(PIN_STATUS_LED, state ? LED_ON : LED_OFF);
}

bool SafetyManager::requestWatering(int duration_sec, const String& reason, float currentWaterLevelCm, int currentHour) {
    if (pump_active_) {
        last_inhibit_reason_ = "Pump is already running";
        return false;
    }

    // 1. Dry Tank Check
    if (currentWaterLevelCm < config_.gauges.water.min_safe_level_cm) {
        last_inhibit_reason_ = "Inhibited: Water reservoir is empty or below safe cutoff (" + 
                               String(currentWaterLevelCm, 1) + "cm < " + 
                               String(config_.gauges.water.min_safe_level_cm, 1) + "cm)";
        return false;
    }

    // 2. Cooldown Delay Check (unless system just booted and last_pump_stop_ms_ == 0)
    if (last_pump_stop_ms_ > 0) {
        uint32_t elapsed_ms = millis() - last_pump_stop_ms_;
        uint32_t cooldown_ms = static_cast<uint32_t>(config_.safety.min_time_between_cycles_sec) * 1000UL;
        if (elapsed_ms < cooldown_ms) {
            uint32_t remaining_sec = (cooldown_ms - elapsed_ms) / 1000UL;
            last_inhibit_reason_ = "Inhibited: Cooldown active (" + String(remaining_sec) + "s remaining)";
            return false;
        }
    }

    // 3. Daily Limit Check
    if (config_.safety.daily_limit_enabled) {
        if ((daily_watering_seconds_ + duration_sec) > static_cast<uint32_t>(config_.safety.max_seconds_per_day)) {
            last_inhibit_reason_ = "Inhibited: Daily limit reached (" + 
                                   String(daily_watering_seconds_) + "/" + 
                                   String(config_.safety.max_seconds_per_day) + "s used)";
            return false;
        }
    }

    // 4. Night Protection Check (if hour is provided >= 0)
    if (config_.safety.night_protection_enabled && currentHour >= 0) {
        int start = config_.safety.night_start_hour;
        int end = config_.safety.night_end_hour;
        bool is_night = false;
        if (start > end) {
            // Span midnight: e.g. 22:00 to 09:00
            is_night = (currentHour >= start || currentHour < end);
        } else {
            is_night = (currentHour >= start && currentHour < end);
        }

        if (is_night) {
            last_inhibit_reason_ = "Inhibited: Night watering protection active (" + String(currentHour) + ":00)";
            return false;
        }
    }

    // 5. Clamp to hard continuous limit
    int max_allowed = config_.safety.max_continuous_pump_sec;
    if (duration_sec > max_allowed) {
        duration_sec = max_allowed;
    }

    // Start Pump
    target_duration_ms_ = static_cast<uint32_t>(duration_sec) * 1000UL;
    pump_start_ms_ = millis();
    pump_active_ = true;
    last_inhibit_reason_ = "None (Active: " + reason + ")";
    
    setPumpGate(true);
    return true;
}

void SafetyManager::stopWatering(const String& reason) {
    if (!pump_active_) return;

    setPumpGate(false);
    pump_active_ = false;

    uint32_t actual_duration_ms = millis() - pump_start_ms_;
    uint32_t actual_sec = actual_duration_ms / 1000UL;
    daily_watering_seconds_ += actual_sec;
    last_pump_stop_ms_ = millis();

    last_inhibit_reason_ = "Stopped: " + reason;
}

void SafetyManager::tick() {
    if (!pump_active_) return;

    uint32_t elapsed_ms = millis() - pump_start_ms_;
    uint32_t hard_limit_ms = static_cast<uint32_t>(config_.safety.max_continuous_pump_sec) * 1000UL;

    // Hard cutoff if target duration reached or runaway max limit reached
    if (elapsed_ms >= target_duration_ms_ || elapsed_ms >= hard_limit_ms) {
        stopWatering("Target duration or safety runtime reached");
    }
}

uint32_t SafetyManager::getActiveRuntimeSec() const {
    if (!pump_active_) return 0;
    return (millis() - pump_start_ms_) / 1000UL;
}

uint32_t SafetyManager::getSecondsUntilNextAllowed() const {
    if (last_pump_stop_ms_ == 0) return 0;
    uint32_t elapsed_ms = millis() - last_pump_stop_ms_;
    uint32_t cooldown_ms = static_cast<uint32_t>(config_.safety.min_time_between_cycles_sec) * 1000UL;
    if (elapsed_ms >= cooldown_ms) return 0;
    return (cooldown_ms - elapsed_ms) / 1000UL;
}

void SafetyManager::resetDailyQuota() {
    daily_watering_seconds_ = 0;
}
