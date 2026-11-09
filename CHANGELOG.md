# Changelog

All notable changes to the Plant ESP32 firmware and web dashboard will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.36] - 2026-11-09

### 🌿 Categorical Horticultural Light Bands & Daily Light Integral (DLI)
- **Categorical Horticultural Light Bands**: Redesigned the ambient light meter to reflect standard horticultural plant lighting tiers: Deep Shade (<250 lx), Low Light (250–1,000 lx), Medium Light (1,000–2,500 lx), Bright Indirect (2,500–5,000 lx), and Direct Sun (>5,000 lx).
- **Non-Linear Gauge Dial**: Replaced linear lux scaling with normalized piecewise band sectors so indoor light readings (such as 108 lx) accurately position within the Deep Shade arc rather than pegging against the zero stop.
- **Daily Light Integral (DLI)**: Added persistent calculation and display of yesterday's total cumulative light in mol/m²/day (`LUX_TO_PPFD_SUNLIGHT` trapezoidal numerical integration) with horticultural quality ratings.
- **Firmware Integration**: `/api/status` now reports `dli_yesterday` directly from historical telemetry readings in flash memory, updating every day at local midnight.

---

## [0.2.35] - 2026-11-09

### 🤖 Autonomous Boot & Post-Midnight OTA Firmware Checking
- **Autonomous Boot Check**: ESP32 firmware now checks GitHub Releases for new firmware ~60 seconds after boot once Wi-Fi and SNTP stabilize.
- **Nightly Post-Midnight Check**: Internal background worker checks GitHub automatically once per day at 02:00 AM local time (using NTP and configured GMT offset).
- **Firmware Status Reporting**: `/api/ota/status` exposes `update_available`, `latest_firmware_version`, and `last_check_unix` directly from the hardware, instantly informing connected browsers without requiring extra client-side queries.
- **Safety**: Autonomous checks only query and flag available updates—they never flash or reboot without explicit user confirmation.

---

## [0.2.34] - 2026-11-09

### ⚡ Service Worker Cache Fix & Instant UI Updates
- **Network-First PWA Strategy**: Replaced stale-while-revalidate caching in `sw.js` with network-first for navigation and script assets, ensuring browser immediately loads updated UI without serving stale cached versions.
- **Auto-Controller Refresh**: Added `controllerchange` listener to automatically reload the page when a new service worker version is activated.
- **Cache Invalidation**: Bumped service worker cache to `plant-esp32-v9` and asset query to `v=21`.

---

## [0.2.33] - 2026-11-09

### 🚀 Daily OTA Checks, Release Review & Update Status Field
- **Pre-Install Release Review**: The update button now reports release notes and changelog details in an interactive modal for the user to review before asking to apply.
- **Update Availability Field**: Added a dedicated field next to the update button that notifies the user when an update is available (`✨ Update Available: vX.Y.Z`) or confirms up-to-date status.
- **Automated Daily Checks**: The system checks GitHub Releases automatically once per day (tracking last check timestamp in localStorage) with background polling and hourly verification.
- **Header Update Pill**: Added an optional update indicator pill next to the version badge in the dashboard header when a new release is available.

---

## [0.2.32] - 2026-11-09

### 🕒 Simplified Real-Time Clock & Background Resync
- **Clean Clock Badge**: Streamlined the live header clock display (`YYYY-MM-DD HH:MM:SS`) by removing drift statistics from the UI.
- **Background Periodic Resync**: Device time syncs from the ESP32 during standard 60-second status updates, with client-side 1-second ticking between polls.
- **Hardware Diagnostics Streamlining**: Diagnostics panel displays clean Device Time and hourly NTP synchronization status without drift overhead.

---

## [0.2.31] - 2026-11-09

### 🕒 Real-Time Clock & Time Drift Tracking, Robust NTP Resync
- **Live Clock in Web Header**: Added live date, time, and drift badge next to the version badge (`YYYY-MM-DD HH:MM:SS (±0.0s)`). Ticks every second on the client machine with zero server request polling.
- **Time Drift Monitoring**: Automatically calculates and tracks drift between the ESP32 hardware RTC and browser device, updating on periodic status fetches.
- **Hourly NTP Resync & Auto-Retry**: Configured lwIP SNTP with explicit hourly resync (`sntp_set_sync_interval(3600000)` = 24 times/day) and added automatic probe retry if un-synchronized at boot.
- **Hardware Diagnostics Time Rows**: Added Device Time, NTP Status, and Time Drift to the Hardware Diagnostics panel.

---

## [0.2.30] - 2026-11-09

### 💾 Persistent Flash Logging & Dedicated Environmental Table Columns
- **Persistent Flash Telemetry**: Sensor readings now persist across reboots and firmware updates via NVS flash storage (`plant_log` partition blob). Historical readings are reloaded into the FIFO ring buffer on boot.
- **Dedicated Log Table Columns**: Replaced cramped sub-labels with dedicated table columns for **Air Temp**, **Humidity**, and **Light** in the Sensor Log view.
- **Updated Capacity Badge**: Log count badge now displays `X / 100 in Flash`.

---

## [0.2.29] - 2026-11-09

### 🌿 Environmental Sensor Settings & Dashboard Stability
- **Settings Tab Relocation**: Relocated the **Optional Environmental Sensors** card (SHT30, BH1750, and LDR) directly into the **⚙️ Settings** tab under System Controls with direct persistence to flash.
- **Client Script Syntax Fix**: Resolved duplicate token block in `app.js` chart drawing routine that prevented JavaScript execution and UI initialization.
- **Cache Invalidation**: Bumped service worker cache and asset bundle to ensure immediate browser pickup.

---

## [0.2.28] - 2026-11-08

### 🌿 SHT30, BH1750, and LDR Environmental Sensors
- **Hardware I2C Driver**: Configured master bus on GPIO 5 (SCL) and GPIO 6 (SDA) supporting SHT30 (Air Temperature °C & Humidity % with CRC-8 verification) and BH1750 (Ambient Light in Lux).
- **LDR Light Sensor**: Analog light measurement on GPIO 1 ADC1 Channel 1 with 10k divider (0–100% illuminance).
- **Settings Toggles**: Added toggles under **Settings -> Optional Environmental Sensors** for SHT30, BH1750, and LDR with persistence to NVS flash.
- **Dynamic Dials & Telemetry Graph**: Added modular circular dials that automatically display when enabled, series toggle buttons on the interactive telemetry chart, and environmental columns in the recent log history.

---

## [0.2.27] - 2026-11-08

### 🏷️ App Display Name Rename
- **Rename PWA App to "Plants"**: Changed web app manifest `name` and `short_name`, as well as Apple mobile web app title meta tag from `PlantESP32` / `Plant Monitor & Waterer` to simply `Plants`.

---

## [0.2.26] - 2026-11-08

### 📱 Android & WebAPK PWA Parity Fixes
- **Manifest ID & Crossorigin Credentials**: Added `"id": "/"` to [`manifest.json`](file:///Users/robertlongbottom/dev/plant-esp32/data/manifest.json) and `crossorigin="use-credentials"` to the manifest link in [`index.html`](file:///Users/robertlongbottom/dev/plant-esp32/data/index.html) to support HTTP Basic Auth behind reverse proxies.
- **Dedicated Maskable Icons**: Separated standard (`"any"`) and adaptive (`"maskable"`) icon declarations for both 192x192 and 512x512 icons matching the proven `plant-p2` configuration.
- **Service Worker Stale-While-Revalidate App Shell**: Upgraded [`sw.js`](file:///Users/robertlongbottom/dev/plant-esp32/data/sw.js) with immediate static app shell pre-caching and stale-while-revalidate fetching, satisfying Chrome Android WebAPK offline installability requirements while strictly excluding `/api/` telemetry routes.

---

## [0.2.20] - 2026-11-08

### 🩹 Web Server Socket Limit Tuning
- **Max Open Sockets Adjustment**: Set `max_open_sockets: 6` in `ServerConfig`. In ESP-IDF, lwIP limits total system sockets to 10 (with 3 consumed internally by HTTP control loops). Setting `max_open_sockets: 6` prevents the `ESP_ERR_INVALID_ARG (258)` bind failure while still providing 50% more socket concurrency than the default 4.

---

## [0.2.19] - 2026-11-08

### 💓 Double-Flash Heartbeat & Web Server Socket Capacity
- **Double-Flash Heartbeat Pattern**: Replaced the single LED blip every 10 seconds with a distinct "thump-thump" double-flash heartbeat pattern (80ms ON / 120ms OFF / 80ms ON / ~9.7s sleep) to intuitively signal that the system is fully healthy and operational.
- **Increased Web Server Socket Capacity**: Increased ESP-IDF `max_open_sockets` from 4 to 8. This ensures modern browsers loading multiple assets simultaneously (`index.html`, `app.js`, `style.css`, `favicon.svg`, `/api/status`, `/api/config`) never exhaust the connection pool or stall.
- **Balanced Default Wi-Fi Power**: Increased default RF transmit power from +13.0 dBm (52) to +17.0 dBm (68, ~50mW) to eliminate packet loss and weak signal dropouts across living room walls while still maintaining lower chip temperatures than the +19.5 dBm maximum.

---

## [0.2.18] - 2026-11-08

### 🩹 Web Server Response Header Hotfix
- **Remove Conflicting `Content-Length` Header**: Removed manually specified `Content-Length` header from `send_file` and `send_json`. In ESP-IDF, setting a manual `Content-Length` header conflicts with `into_response()` and `write_all()` (which use ESP-IDF chunked transfer encoding), causing large responses (like `index.html` and full `config.json`) to abort.
- **Retain Clean `Connection: close`**: Retained `("Connection", "close")` to reliably instruct browsers not to hold open idle TCP sessions.

---

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
