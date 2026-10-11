#include "web_server.h"
#include <ESPmDNS.h>
#include <ArduinoOTA.h>
#include <LittleFS.h>
#include <WiFi.h>
#include "config_manager.h"

WebServerManager::WebServerManager(AppConfig& config,
                                   SensorManager& sensorMgr,
                                   SafetyManager& safetyMgr,
                                   SensorRingBuffer<100>& ringBuffer)
    : config_(config),
      sensorMgr_(sensorMgr),
      safetyMgr_(safetyMgr),
      ringBuffer_(ringBuffer) {}

void WebServerManager::begin() {
    // 1. Enable Global CORS for Reverse Proxy / External Tools
    DefaultHeaders::Instance().addHeader("Access-Control-Allow-Origin", "*");
    DefaultHeaders::Instance().addHeader("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
    DefaultHeaders::Instance().addHeader("Access-Control-Allow-Headers", "*");

    setupWebSocket();
    setupRoutes();
    setupMDNS();
    setupOTA();

    server_.begin();
    Serial.println("[HTTP] Async Web Server started on port 80");
}

void WebServerManager::setupWebSocket() {
    ws_.onEvent([](AsyncWebSocket* server, AsyncWebSocketClient* client, AwsEventType type, void* arg, uint8_t* data, size_t len) {
        if (type == WS_EVT_CONNECT) {
            Serial.printf("[WS] Client #%u connected from %s\n", client->id(), client->remoteIP().toString().c_str());
        } else if (type == WS_EVT_DISCONNECT) {
            Serial.printf("[WS] Client #%u disconnected\n", client->id());
        }
    });
    server_.addHandler(&ws_);
}

void WebServerManager::setupMDNS() {
    const char* host = config_.system.hostname.length() > 0 ? config_.system.hostname.c_str() : "plants";
    if (MDNS.begin(host)) {
        MDNS.addService("http", "tcp", 80);
        Serial.printf("[mDNS] Responder started: http://%s.local\n", host);
    } else {
        Serial.println("[mDNS] Error setting up mDNS responder");
    }
}

void WebServerManager::setupOTA() {
    ArduinoOTA.setHostname(config_.system.hostname.c_str());
    ArduinoOTA.onStart([]() {
        Serial.println("[OTA] Firmware update starting...");
    });
    ArduinoOTA.onEnd([]() {
        Serial.println("\n[OTA] Firmware update complete. Rebooting...");
    });
    ArduinoOTA.onProgress([](unsigned int progress, unsigned int total) {
        Serial.printf("[OTA] Progress: %u%%\r", (progress / (total / 100)));
    });
    ArduinoOTA.onError([](ota_error_t error) {
        Serial.printf("[OTA] Error[%u]: ", error);
    });
    ArduinoOTA.begin();
    Serial.println("[OTA] Over-The-Air firmware updater ready");
}

void WebServerManager::setupRoutes() {
    // Static Files from LittleFS
    server_.serveStatic("/", LittleFS, "/").setDefaultFile("index.html");

    // Pre-flight OPTIONS handler for CORS
    server_.onNotFound([](AsyncWebServerRequest* request) {
        if (request->method() == HTTP_OPTIONS) {
            request->send(204);
        } else {
            request->send(404, "text/plain", "Not found");
        }
    });

    // 1. GET /api/status
    server_.on("/api/status", HTTP_GET, [this](AsyncWebServerRequest* request) {
        this->handleStatus(request);
    });

    // 2. GET /api/sensor-log (with ?n= query parameter)
    server_.on("/api/sensor-log", HTTP_GET, [this](AsyncWebServerRequest* request) {
        this->handleSensorLog(request);
    });

    // 3. GET /api/config
    server_.on("/api/config", HTTP_GET, [this](AsyncWebServerRequest* request) {
        this->handleGetConfig(request);
    });

    // 4. POST /api/config
    server_.on("/api/config", HTTP_POST, [](AsyncWebServerRequest* request) {}, NULL,
        [this](AsyncWebServerRequest* request, uint8_t* data, size_t len, size_t index, size_t total) {
            JsonDocument doc;
            DeserializationError err = deserializeJson(doc, data, len);
            if (err) {
                request->send(400, "application/json", "{\"error\":\"Invalid JSON\"}");
                return;
            }

            // Update configuration from incoming document
            if (doc["gauges"]["soil_moisture"]["trigger_threshold"]) {
                this->config_.gauges.soil.trigger_threshold = doc["gauges"]["soil_moisture"]["trigger_threshold"].as<int>();
            }
            if (doc["safety"]["max_continuous_pump_sec"]) {
                this->config_.safety.max_continuous_pump_sec = doc["safety"]["max_continuous_pump_sec"].as<int>();
            }
            if (doc["safety"]["min_time_between_cycles_sec"]) {
                this->config_.safety.min_time_between_cycles_sec = doc["safety"]["min_time_between_cycles_sec"].as<int>();
            }

            ConfigManager::saveConfig(this->config_);
            this->sensorMgr_.updateConfig(this->config_);
            this->safetyMgr_.updateConfig(this->config_);

            request->send(200, "application/json", "{\"status\":\"success\"}");
        });

    // 5. POST /api/water (Manual Watering Trigger)
    server_.on("/api/water", HTTP_POST, [](AsyncWebServerRequest* request) {}, NULL,
        [this](AsyncWebServerRequest* request, uint8_t* data, size_t len, size_t index, size_t total) {
            int duration = this->config_.timing.manual_water_duration_sec;
            if (len > 0) {
                JsonDocument doc;
                if (!deserializeJson(doc, data, len) && doc["duration"]) {
                    duration = doc["duration"].as<int>();
                }
            }

            // Fetch live distance/water level for safety dry-tank check
            float distance = this->sensorMgr_.readUltrasonicDistanceCm();
            float water_level = this->config_.gauges.water.tank_height_cm - distance;

            bool ok = this->safetyMgr_.requestWatering(duration, "Manual Web Request", water_level, -1);
            JsonDocument res;
            res["success"] = ok;
            res["message"] = ok ? "Watering started" : this->safetyMgr_.getLastInhibitReason();
            res["duration_sec"] = duration;

            String out;
            serializeJson(res, out);
            request->send(ok ? 200 : 409, "application/json", out);
        });

    // 6. GET /api/sensor/health
    server_.on("/api/sensor/health", HTTP_GET, [this](AsyncWebServerRequest* request) {
        this->handleHealth(request);
    });

    // 7. POST /api/sensor/ultrasonic/reset
    server_.on("/api/sensor/ultrasonic/reset", HTTP_POST, [this](AsyncWebServerRequest* request) {
        this->handleUltrasonicReset(request);
    });
}

void WebServerManager::handleStatus(AsyncWebServerRequest* request) {
    SensorReading latest;
    auto recent = ringBuffer_.getRecent(1);
    if (!recent.empty()) {
        latest = recent[0];
    }

    JsonDocument doc;
    doc["hostname"] = config_.system.hostname;
    doc["ip"] = WiFi.localIP().toString();
    doc["mdns"] = config_.system.hostname + ".local";
    doc["uptime_sec"] = millis() / 1000UL;
    doc["wifi_rssi"] = WiFi.RSSI();

    JsonObject s = doc["sensors"].to<JsonObject>();
    s["soil_percent"] = roundf(latest.soil_percent * 10.0f) / 10.0f;
    s["soil_raw"] = latest.soil_raw_adc;
    s["water_level_cm"] = roundf(latest.water_level_cm * 10.0f) / 10.0f;
    s["water_percent"] = roundf(latest.water_percent * 10.0f) / 10.0f;
    s["distance_cm"] = roundf(latest.distance_cm * 10.0f) / 10.0f;
    s["supply_voltage"] = roundf(latest.supply_voltage * 100.0f) / 100.0f;
    s["chip_temp"] = roundf(latest.chip_temp * 10.0f) / 10.0f;

    JsonObject p = doc["pump"].to<JsonObject>();
    p["active"] = safetyMgr_.isPumpActive();
    p["runtime_sec"] = safetyMgr_.getActiveRuntimeSec();
    p["daily_used_sec"] = safetyMgr_.getDailyRuntimeSec();
    p["cooldown_remaining_sec"] = safetyMgr_.getSecondsUntilNextAllowed();
    p["status_message"] = safetyMgr_.getLastInhibitReason();

    String out;
    serializeJson(doc, out);
    request->send(200, "application/json", out);
}

void WebServerManager::handleSensorLog(AsyncWebServerRequest* request) {
    size_t n = 10;
    if (request->hasParam("n")) {
        n = static_cast<size_t>(request->getParam("n")->value().toInt());
        if (n == 0) n = 10;
        if (n > 100) n = 100;
    }

    JsonDocument doc;
    JsonArray arr = doc.to<JsonArray>();
    ringBuffer_.serializeToJson(arr, n);

    String out;
    serializeJson(doc, out);
    request->send(200, "application/json", out);
}

void WebServerManager::handleGetConfig(AsyncWebServerRequest* request) {
    if (LittleFS.exists("/config.json")) {
        request->send(LittleFS, "/config.json", "application/json");
    } else {
        request->send(404, "application/json", "{\"error\":\"config.json missing\"}");
    }
}

void WebServerManager::handleHealth(AsyncWebServerRequest* request) {
    JsonDocument doc;
    doc["preventive_resets"] = sensorMgr_.getPreventiveResetCount();
    doc["adc_calibration"] = "eFuse factory curve active";
    doc["wifi_rssi"] = WiFi.RSSI();
    doc["free_heap_bytes"] = ESP.getFreeHeap();
    doc["chip_temp_c"] = sensorMgr_.readChipTemperature();

    String out;
    serializeJson(doc, out);
    request->send(200, "application/json", out);
}

void WebServerManager::handleUltrasonicReset(AsyncWebServerRequest* request) {
    sensorMgr_.forceUltrasonicReset();
    JsonDocument doc;
    doc["success"] = true;
    doc["preventive_resets"] = sensorMgr_.getPreventiveResetCount();

    String out;
    serializeJson(doc, out);
    request->send(200, "application/json", out);
}

void WebServerManager::broadcastTelemetry(const SensorReading& reading) {
    if (ws_.count() == 0) return;

    JsonDocument doc;
    doc["type"] = "telemetry";
    doc["soil_percent"] = roundf(reading.soil_percent * 10.0f) / 10.0f;
    doc["soil_raw"] = reading.soil_raw_adc;
    doc["water_level_cm"] = roundf(reading.water_level_cm * 10.0f) / 10.0f;
    doc["water_percent"] = roundf(reading.water_percent * 10.0f) / 10.0f;
    doc["supply_voltage"] = roundf(reading.supply_voltage * 100.0f) / 100.0f;
    doc["chip_temp"] = roundf(reading.chip_temp * 10.0f) / 10.0f;
    doc["pump_active"] = reading.pump_active;

    String out;
    serializeJson(doc, out);
    ws_.textAll(out);
}

void WebServerManager::loop() {
    ArduinoOTA.handle();
    ws_.cleanupClients();
}
