#include <Arduino.h>
#include <WiFi.h>
#include <esp_task_wdt.h>
#include <time.h>

#include "pin_definitions.h"
#include "config_schema.h"
#include "config_manager.h"
#include "ring_buffer.h"
#include "sensor_manager.h"
#include "safety_manager.h"
#include "web_server.h"

#if __has_include("secrets.h")
#include "secrets.h"
#else
#define WIFI_SSID "YOUR_WIFI_SSID"
#define WIFI_PASSWORD "YOUR_WIFI_PASSWORD"
#endif

// =============================================================================
// Global Application State & Managers
// =============================================================================
AppConfig appConfig;
SensorRingBuffer<100> sensorHistory;
SensorManager sensorMgr(appConfig);
SafetyManager safetyMgr(appConfig);
WebServerManager webServerMgr(appConfig, sensorMgr, safetyMgr, sensorHistory);

// Task Handles
TaskHandle_t hSafetyTask = nullptr;
TaskHandle_t hSensorTask = nullptr;
TaskHandle_t hWebTask = nullptr;

// =============================================================================
// Helper: NTP Time & Midnight Daily Reset
// =============================================================================
int getCurrentLocalHour() {
    struct tm timeinfo;
    if (getLocalTime(&timeinfo, 50)) {
        return timeinfo.tm_hour;
    }
    return -1; // Time not yet synchronized
}

void checkMidnightReset() {
    static int last_day = -1;
    struct tm timeinfo;
    if (getLocalTime(&timeinfo, 50)) {
        if (last_day != -1 && timeinfo.tm_mday != last_day) {
            Serial.println("[NTP] Midnight reached: resetting daily watering quota");
            safetyMgr.resetDailyQuota();
        }
        last_day = timeinfo.tm_mday;
    }
}

// =============================================================================
// FreeRTOS Task 1: Safety & Pump Supervisor (Priority: HIGH)
// =============================================================================
void taskSafetySupervisor(void* pvParameters) {
    TickType_t xLastWakeTime = xTaskGetTickCount();
    const TickType_t xFrequency = pdMS_TO_TICKS(50); // 20 Hz update rate

    for (;;) {
        safetyMgr.tick();
        vTaskDelayUntil(&xLastWakeTime, xFrequency);
    }
}

// =============================================================================
// FreeRTOS Task 2: Sensor Acquisition & Auto-Watering (Priority: NORMAL)
// =============================================================================
void taskSensorAcquisition(void* pvParameters) {
    // Initial startup delay to let power supplies and sensors stabilize
    vTaskDelay(pdMS_TO_TICKS(appConfig.timing.startup_delay_sec * 1000UL));

    for (;;) {
        bool pumpRunning = safetyMgr.isPumpActive();
        SensorReading reading = sensorMgr.readAllSensors(pumpRunning);

        // Store reading in circular RAM ring buffer
        sensorHistory.push(reading);

        // Broadcast telemetry via WebSockets to connected browsers
        webServerMgr.broadcastTelemetry(reading);

        Serial.printf("[SENSOR] Soil: %.1f%% | Water: %.1fcm | 5V Rail: %.2fV | Bat: %.2fV | Chip: %.1fC\n",
                      reading.soil_percent, reading.water_level_cm, reading.supply_voltage,
                      reading.battery_voltage, reading.chip_temp);

        // Automated Watering Logic
        if (!pumpRunning) {
            float soil = reading.soil_percent;
            int threshold = appConfig.gauges.soil.trigger_threshold;

            if (soil < threshold && reading.water_level_cm >= appConfig.gauges.water.min_safe_level_cm) {
                int hour = getCurrentLocalHour();
                int duration = appConfig.timing.automated_water_duration_sec;

                bool started = safetyMgr.requestWatering(duration, "Soil dry trigger", reading.water_level_cm, hour);
                if (started) {
                    Serial.printf("[AUTO] Low soil moisture (%.1f%% < %d%%). Started watering for %ds\n", soil, threshold, duration);
                }
            }
        }

        // Delay until next acquisition cycle
        int interval_sec = appConfig.timing.sensor_read_interval_sec;
        if (interval_sec < 5) interval_sec = 5;
        vTaskDelay(pdMS_TO_TICKS(interval_sec * 1000UL));
    }
}

// =============================================================================
// FreeRTOS Task 3: Network, Web Server & NTP (Priority: LOW)
// =============================================================================
void taskNetworkAndWeb(void* pvParameters) {
    uint32_t last_ntp_check_ms = 0;

    for (;;) {
        // Run web server maintenance (OTA handles, WebSockets ping/pong)
        webServerMgr.loop();

        // Check Wi-Fi connection and reconnect if lost
        if (WiFi.status() != WL_CONNECTED) {
            Serial.println("[WIFI] Reconnecting...");
            WiFi.reconnect();
            vTaskDelay(pdMS_TO_TICKS(5000));
        }

        // Daily midnight check for watering quota reset
        if (millis() - last_ntp_check_ms > 60000UL) {
            checkMidnightReset();
            last_ntp_check_ms = millis();
        }

        vTaskDelay(pdMS_TO_TICKS(10));
    }
}

// =============================================================================
// System Setup
// =============================================================================
void setup() {
    Serial.begin(115200);
    delay(1000);
    Serial.println("\n============================================");
    Serial.println("   Plant Monitor ESP32-C3 SuperMini");
    Serial.println("============================================");

    // 0. Low-Power Optimization: Downclock to 80 MHz (cuts core power roughly in half)
    setCpuFrequencyMhz(80);
    Serial.printf("[PWR] CPU Frequency set to %u MHz\n", getCpuFrequencyMhz());

    // 1. Load configuration from LittleFS
    ConfigManager::loadConfig(appConfig);

    // 2. Initialize hardware managers
    sensorMgr.begin();
    safetyMgr.begin();

    // 3. Connect to Wi-Fi with Modem Sleep enabled
    Serial.printf("[WIFI] Connecting to '%s'...\n", WIFI_SSID);
    WiFi.mode(WIFI_STA);
    WiFi.setSleep(WIFI_PS_MIN_MODEM); // Power down radio between router DTIM beacon intervals
    WiFi.setHostname(appConfig.system.hostname.c_str());
    WiFi.begin(WIFI_SSID, WIFI_PASSWORD);
    Serial.println("[PWR] Wi-Fi Modem Sleep enabled (WIFI_PS_MIN_MODEM)");

    // Wait up to 10 seconds for Wi-Fi connection
    uint32_t wifi_start = millis();
    while (WiFi.status() != WL_CONNECTED && (millis() - wifi_start < 10000)) {
        delay(250);
        Serial.print(".");
    }
    Serial.println();

    if (WiFi.status() == WL_CONNECTED) {
        Serial.printf("[WIFI] Connected! IP Address: %s\n", WiFi.localIP().toString().c_str());
    } else {
        Serial.println("[WIFI] Initial connection timed out. Running offline mode / will retry in background.");
    }

    // 4. Initialize NTP time
    if (!appConfig.system.ntp_servers.empty()) {
        configTime(appConfig.system.gmt_offset * 3600, 0, 
                   appConfig.system.ntp_servers[0].c_str(),
                   appConfig.system.ntp_servers.size() > 1 ? appConfig.system.ntp_servers[1].c_str() : nullptr);
    }

    // 5. Start Web Server, mDNS & OTA
    webServerMgr.begin();

    // 6. Launch FreeRTOS Preemptive Tasks
    xTaskCreate(
        taskSafetySupervisor,
        "SafetyTask",
        4096,
        nullptr,
        3, // Priority HIGH
        &hSafetyTask
    );

    xTaskCreate(
        taskSensorAcquisition,
        "SensorTask",
        4096,
        nullptr,
        2, // Priority NORMAL
        &hSensorTask
    );

    xTaskCreate(
        taskNetworkAndWeb,
        "WebTask",
        6144,
        nullptr,
        1, // Priority LOW
        &hWebTask
    );

    Serial.println("[SYS] FreeRTOS tasks started. System running.");
}

void loop() {
    // Empty: all tasks run under FreeRTOS scheduler
    vTaskDelay(pdMS_TO_TICKS(1000));
}
