# Changelog

All notable changes to the Plant ESP32 firmware and web dashboard will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.17] - 2026-11-08

### 📶 Wi-Fi Radio Power Controls & Web Server Keep-Alive Hardening
- **Wi-Fi Radio Settings Card**: Added configurable Wi-Fi settings in the Web UI Settings tab, committed directly to NVS flash:
  - **RF Transmit Power Selector**: Choose between +11 dBm (lowest heat / same room, ~13mW), +13 dBm (balanced / recommended, ~20mW), +15 dBm, +17 dBm, and +19.5 dBm (max power, ~100mW).
  - **Modem Sleep Toggle**: Configurable DTIM power saving (`WIFI_PS_MIN_MODEM`) or zero-sleep (`WIFI_PS_NONE`) for rock-solid zero-latency mesh connectivity.
  - **20 MHz Bandwidth Lock (HT20)**: Option to lock the 2.4 GHz station to 20 MHz bandwidth to halve baseband synthesis overhead and lower thermals without affecting any other devices on the network.
- **Dynamic Radio Configuration**: Updates from the Settings page apply immediately to the Wi-Fi PHY without requiring a reboot.
- **Web Server TCP Socket Exhaustion Fix**: Added explicit `Connection: close` and `Content-Length` headers to all static file and JSON responses. Prevents browser HTTP/1.1 keep-alive from exhausting the ESP32's limited socket pool (which previously caused UI timeouts and perceived disconnects).

---

## [0.2.16] - 2026-11-07

### 📈 Dynamic Y-Axis Auto-Scale for Telemetry Chart
- **Dynamic Auto-Scale Toggle**: Added an interactive `📐 Auto-Scale` toggle button on the Telemetry Trends card header to toggle between Fixed and Dynamic Auto-Scale modes, persisted in `localStorage`.
- **Intelligent Multi-Sensor Scaling**:
  - **Single Sensor / Matching Units (e.g. Chip Temp °C, or Supply & Battery V)**: Auto-scales Y-axis ticks and grid lines directly to the actual min-max range with 12% padding, showing real unit labels (e.g., `45.8°C`, `46.5°C`, `47.2°C`) instead of flatlines on a generic 0–100% axis.
  - **Mixed Sensor Combinations (e.g. Soil % + Temp °C + Water cm)**: Dynamically scales each active sensor curve independently to fill the vertical canvas space, allowing simultaneous visualization and correlation of micro-trends across all metrics, with an informative scale badge.
  - **Fixed Mode Refinement**: Even in Fixed mode, when a single unit is selected, displays true physical unit ticks (`20–80°C`, `0–30 cm`, `0–6V`) rather than a percentage.

---

## [0.2.15] - 2026-11-07

### ❄️ Power & Thermal Optimization
- **Re-enable Wi-Fi Modem Sleep**: Enabled `WIFI_PS_MIN_MODEM` once connected in Station mode to allow the RF receiver and PLL to sleep between DTIM beacons, reducing continuous power draw and cooling silicon operating temperatures from ~65°C back towards ~44°C.
- **Robust Connection Handshake**: Retained zero-sleep (`WIFI_PS_NONE`) during association and DHCP negotiation to prevent packet drops and router timeouts during initial connection.

---

## [0.2.14] - 2026-11-07

### ⏱️ Telemetry Timestamps & SNTP Epoch
- **Table Timestamps**: Replaced raw relative seconds (`+4840s`) in the telemetry log table with formatted local time (`14:48:30`) and relative elapsed badges (`10m ago`).
- **Interactive Chart & Tooltip**: Telemetry trends X-axis and hover tooltips now display local time timestamps instead of raw seconds.
- **SNTP Epoch Storage**: Sensor readings record true UNIX epoch timestamps when NTP is synchronized, falling back seamlessly to boot uptime.

---

## [0.2.13] - 2026-11-07

### 🏷️ UI & Title Branding Update
- **Dashboard Title**: Renamed application header and browser title from `Plant Monitor P2` to `Plant Monitor & Waterer`.
- **Wireless OTA Verification**: End-to-end testing of dual-partition OTA deployment.

---

## [0.2.12] - 2026-11-07

### 🔄 Dual-OTA Hardware Partitioning & Validation
- **Partition Verification**: Verified dual-OTA hardware partition layout (`ota_0` and `ota_1`, 1984 KB per slot) initialized in flash.
- **OTA Diagnostic Reporting**: Improved `get_ota_status()` fallback logic to accurately distinguish between active OTA standby slots and unpartitioned flash.
- **Diagnostic UI Feedback**: Updated web dashboard settings panel to clearly report partition status.

---

## [0.2.11] - 2026-11-07

### 📶 Wi-Fi Mesh Stability Enhancements (ESPEasy Best Practices)
- **Zero-Sleep Wi-Fi (`WIFI_PS_NONE`)**: Disabled modem sleep 24/7. Prevents mesh routers (e.g. TP-Link Deco) from dropping unicast frames directed to sleeping stations.
- **Connection Settling Delay**: Added a 750ms settling pause after WPA2 association before DHCP lease negotiation to avoid router handshake timing bugs.
- **Periodic Gratuitous Heartbeat (Task 5)**: Broadcasts keep-alive frames every 30s to keep the station active in router ARP and bridge port tables so the device never drops off the network.
- **Live Signal Strength (RSSI)**: Network watchdog queries the hardware driver directly to display real-time Wi-Fi signal strength in dBm on the dashboard.
- **Self-Healing Watchdog**: Automatically initiates a clean reboot if Wi-Fi connection is continuously lost for >4 minutes.

### 🌐 Dashboard UI & Changelog
- **Clickable Links in OTA Modal**: Markdown links in GitHub release notes are now rendered as clickable links (`target="_blank"`), and release highlights display as clean bulleted lists.
- **Automated Release Summaries**: Release workflow automatically extracts curated highlights from `CHANGELOG.md`.

---

## [0.2.10] - 2026-11-07

### 🛡️ Safety & Bug Fixes
- **Daily Watered Overflow Fix**: Replaced raw unsigned subtraction with clamped `elapsed_millis` across all pump safety timers. Prevents an unsigned wrap when scheduling watering requests, which previously caused the daily counter to jump to 4,294,966s and lock out watering.
- **Supervisor Timestamp Race**: Sampled `now_ms` inside the `SafetySupervisor` mutex lock to ensure the supervisor never evaluates against an out-of-order timestamp.
- **Real-Time Clock (NTP) Sync**: Fixed `local_hour()` and `local_day()` to use `SystemTime::now()` rather than boot uptime. Ensures local hour checks and automatic midnight quota resets operate accurately.
- **SNTP Service Persistence**: Kept `EspSntp` active in memory across the station connection lifecycle.
- **Wi-Fi Handshake & Power Boost**: Set `WIFI_PS_NONE` during association and configured RF transmit power to maximum (+19.5 dBm / 78) to eliminate packet loss with mesh access points (TP-Link Deco).

### ✨ Features
- **Night Quiet Hours Indicator**: Added a real-time status display in the dashboard *Watering & Safety Supervisor* panel showing whether night protection is *Active*, *Inactive*, or *Disabled* along with the active time window.
- **Configurable Quiet Hours**: Added settings controls in Section 5 to configure the quiet hours start/end times (0–23) and toggle protection on or off.

---

## [0.2.9] - 2026-11-06

### ✨ Features
- **Last Watered Summary**: Added human-readable relative time and duration (e.g. `15m ago (5s)`) to the dashboard and API.
- **Dual OTA Flash Partitions**: Expanded OTA partition slots to 1984 KB for reliable in-field updates.
- **Size Optimizations**: Enabled LTO and size optimization flags with automated CI OTA partition guardrails.

---

## [0.2.8] - 2026-11-05

### 🚀 Initial Rust Firmware Release
- Complete memory-safe port of Arduino/C++ firmware to ESP-IDF Rust.
- Multi-task FreeRTOS architecture with prioritized Safety Supervisor (20 Hz), Sensor Acquisition (0.1 Hz), and Visual LED Supervisor.
- Embedded HTTP REST API and WebSocket live telemetry.
- Non-blocking circular RAM ring buffer storing the 100 most recent sensor readings.
