#pragma once

#include <Arduino.h>
#include <ESPAsyncWebServer.h>
#include "config_schema.h"
#include "ring_buffer.h"
#include "sensor_manager.h"
#include "safety_manager.h"

/**
 * @file web_server.h
 * @brief Asynchronous web server handling REST API, WebSockets telemetry, mDNS, and LittleFS static files.
 */

class WebServerManager {
public:
    WebServerManager(AppConfig& config,
                     SensorManager& sensorMgr,
                     SafetyManager& safetyMgr,
                     SensorRingBuffer<100>& ringBuffer);

    void begin();
    void loop(); // Periodic maintenance (mDNS, WebSockets cleanup, etc.)
    void broadcastTelemetry(const SensorReading& reading);

private:
    AppConfig& config_;
    SensorManager& sensorMgr_;
    SafetyManager& safetyMgr_;
    SensorRingBuffer<100>& ringBuffer_;

    AsyncWebServer server_{80};
    AsyncWebSocket ws_{"/ws"};

    void setupRoutes();
    void setupWebSocket();
    void setupMDNS();
    void setupOTA();

    // Route handlers
    void handleStatus(AsyncWebServerRequest* request);
    void handleSensorLog(AsyncWebServerRequest* request);
    void handleGetConfig(AsyncWebServerRequest* request);
    void handleHealth(AsyncWebServerRequest* request);
    void handleUltrasonicReset(AsyncWebServerRequest* request);
};
