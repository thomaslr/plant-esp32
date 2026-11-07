# Changelog

All notable changes to the Plant ESP32 firmware and web dashboard will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
