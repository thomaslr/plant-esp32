# 🌿 Plant Monitor & Automated Irrigation Controller (ESP32-C3 Rust)

[![Build & Release Firmware](https://github.com/thomaslr/plant-esp32/actions/workflows/build-and-release.yml/badge.svg)](https://github.com/thomaslr/plant-esp32/actions/workflows/build-and-release.yml)
[![Rust](https://img.shields.io/badge/rust-nightly-orange.svg)](https://www.rust-lang.org/)
[![Target](https://img.shields.io/badge/target-riscv32imc--esp--espidf-blue.svg)](https://github.com/esp-rs/esp-idf-template)
[![Platform](https://img.shields.io/badge/hardware-ESP32--C3%20SuperMini-brightgreen.svg)](https://lastminuteengineers.com/esp32-c3-super-mini-pinout-reference/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

An industrial-grade, memory-safe automated plant watering and environmental monitoring system built for the ultra-compact **ESP32-C3 SuperMini** (32-bit RISC-V) running on **ESP-IDF** and written in **bare-metal Rust**.

Features:
- **Zero Heap Fragmentation & Crash-Resilience:** Complete domain logic written in memory-safe Rust with 66+ unit tests passing on host (`cargo test`).
- **Dual-Slot A/B Over-The-Air (OTA) Updates:** Dual 1984 KB partitions (`ota_0` / `ota_1`) with hardware bootloader rollback (`CONFIG_BOOTLOADER_APP_ROLLBACK_ENABLE`). If a newly flashed firmware encounters an error, the chip automatically reverts to the previous working slot.
- **Embedded Web Dashboard & PWA:** Self-hosted interactive web dashboard embedded directly in flash via `include_str!`, installable as an offline-capable PWA on iOS, Android, and Desktop.
- **Fail-Safe Plant Protection:** Multi-tiered safety supervisor enforcing daily watering limits, minimum cycle cooldowns, dry tank cutoffs, and night-time quiet hours.
- **Self-Healing Wi-Fi Provisioning:** Captive portal (`PlantMonitor-Setup-XXXX`) with a 5-minute watchdog recovery timer that auto-reconnects when home routers recover after outages.
- **Deterministic Multi-Fault LED Supervisor:** Non-blocking blue LED telemetry signaling Wi-Fi status, pump state, active faults, and flash writing strobes.

---

## 🚀 Quick Start & Installation

Ready to get started? Check out our dedicated step-by-step guide:

👉 **[Complete Quick Start & Flashing Guide (QUICKSTART.md)](QUICKSTART.md)**

### At a Glance: Zero-Install Pre-Built Binaries

Every GitHub release includes two binary formats on the **[Releases Page](https://github.com/thomaslr/plant-esp32/releases)**:

| Binary File | Flash Address | Purpose & Usage |
| :--- | :---: | :--- |
| **`plant-firmware-factory.bin`** | **`0x0`** | **Initial Setup / First-Time Flashing:** Merged factory binary with ESP32-C3 bootloader, Dual-OTA partition table (`ota_0` and `ota_1`), and application code. Flashes blank chips in one shot via Chrome/Edge Web Flasher or USB CLI. |
| **`plant-firmware.bin`** | OTA Partition | **Wi-Fi Updates (OTA):** Standalone application payload uploaded via the web interface (`http://plants1.local`) or downloaded automatically during remote updates. |

* **1-Click Browser Flashing:** Open [Espressif Web Flasher](https://espressif.github.io/esptool-js/) in Chrome/Edge, connect your ESP32-C3 SuperMini at `460800` baud, and flash `plant-firmware-factory.bin` at `0x0`.
* **Single-Command CLI:** `esptool.py --chip esp32c3 write_flash 0x0 plant-firmware-factory.bin`
* **First-Time Wi-Fi Setup:** Connect to the `PlantMonitor-Setup-XXXX` hotspot at `http://192.168.4.1` to configure your Wi-Fi credentials.

See **[QUICKSTART.md](QUICKSTART.md)** for complete hardware checklists, bootloader button timings, and troubleshooting.

---

## 📌 Hardware Pinout Reference (ESP32-C3 SuperMini)

Canonical Pinout Reference: [LastMinuteEngineers ESP32-C3 SuperMini Pinout](https://lastminuteengineers.com/esp32-c3-super-mini-pinout-reference/)

| ESP32-C3 Pin | Connected Component | Function & Conditioning | Mode |
| :--- | :--- | :--- | :--- |
| **GPIO 0** | 18650 Battery Monitor | ADC1_CH0 via 100kΩ/100kΩ divider (R8/R9) + 100nF (C6) | Analog IN |
| **GPIO 1** | Ambient Light Sensor | ADC1_CH1 via LDR1 + 10kΩ divider (R7) + 100nF (C4) | Analog IN |
| **GPIO 2** | Strapping Protected | **Unconnected / High-Z** (Zero boot hangs) | High-Z |
| **GPIO 3** | Capacitive Soil Sensor | ADC1_CH3 connected to TB2 Pin 1 (`AOUT`) + 100nF filter (C2) | Analog IN |
| **GPIO 4** | 5V Supply Rail Monitor | ADC1_CH4 via 100kΩ/100kΩ divider (R1/R2) + 100nF filter (C1) | Analog IN |
| **GPIO 5** | I2C SCL | I2C Clock bus for optional SHT30 / BH1750 modules | Open Drain |
| **GPIO 6** | I2C SDA | I2C Data bus for optional SHT30 / BH1750 modules | Open Drain |
| **GPIO 7** | Ultrasonic Sensor `TRIG`| 10 µs trigger pulse output to HC-SR04 (TB1 Pin 2) | Digital OUT |
| **GPIO 8** | Onboard Status LED | Active LOW blue LED (Active: 0, Inactive: 1) | Digital OUT |
| **GPIO 9** | BOOT Button | Onboard tactile button (pulls LOW when pressed) | Digital IN |
| **GPIO 10** | Pump MOSFET Gate | Digital output to AO3400A Gate via 1kΩ (R5) + 100kΩ pull-down (R6) | Digital OUT |
| **GPIO 20** | Ultrasonic Sensor `ECHO`| Digital input via 1kΩ/2kΩ divider (R3/R4) stepping 5V down to 3.3V | Digital IN |
| **GPIO 21** | Switched Soil Sensor VCC | Digital switched 3.3V power to TB2 Pin 2 (Duty-cycled reading) | Digital OUT |

---

## 🛡️ Multi-Tiered Safety Supervisor

The watering pump can **never** physically run unless all safety invariants are satisfied simultaneously:
1. **Hardware Watchdog Timer (WDT):** 5-second hardware timeout resets the CPU if the scheduler stalls, pulling GPIO 10 LOW via R6 (100kΩ).
2. **Maximum Continuous Pump Runtime:** Hard cutoff after 60 seconds prevents tank emptying during pipe fractures.
3. **Minimum Cycle Cooldown:** Mandatory 300-second pause between watering events allows water absorption into soil substrate.
4. **Daily Cumulative Watering Limit:** Prevents root drowning by capping total daily watering duration (e.g. 93 seconds/day max), resetting at local midnight via SNTP.
5. **Night-Watering Protection:** Automated watering is inhibited during night quiet hours (default: 10:00 PM to 9:00 AM).
6. **Dry Tank Interlock:** Blocks automated and manual pump activation when reservoir water level is below 2.0 cm.

---

## 🔄 Dual-Slot Over-The-Air (OTA) Updates

The ESP32-C3 uses a custom 4MB partition table (`plant-firmware/partitions.csv`):
```text
nvs,      data, nvs,     0x9000,   0x6000,   # 24 KB User config & Wi-Fi credentials
otadata,  data, ota,     0xf000,   0x2000,   # 8 KB Active boot slot pointers (A/B state)
phy_init, data, phy,     0x11000,  0x1000,   # 4 KB RF calibration data
ota_0,    app,  ota_0,   0x20000,  0x1f0000, # 1984 KB Application Slot 0 (Active / Standby)
ota_1,    app,  ota_1,   0x210000, 0x1f0000, # 1984 KB Application Slot 1 (Active / Standby)
```

- When an update is initiated from the Web Dashboard or REST API, the firmware streams `plant-firmware.bin` from GitHub Releases into the inactive slot (`ota_0` or `ota_1`).
- On reboot, the bootloader attempts to boot the new slot. If it crashes or fails to call `mark_running_slot_valid()`, the bootloader automatically rolls back to the previous known-good firmware slot.

---

## 💡 Status LED Signal Reference

The onboard blue LED (GPIO 8) on the ESP32-C3 SuperMini provides real-time visual feedback:

| LED Pattern | Meaning & State |
| :--- | :--- |
| **💙 Single 100ms Pulse (every 10s)** | **Normal Healthy Idle:** All sensors within bounds, Wi-Fi connected. |
| **🌊 Solid ON** | **Watering Active:** Pump motor is running. |
| **⚡ 10 Hz Strobe (Continuous)** | **OTA Flashing:** Flash memory write in progress — do not unplug. |
| **📶 200ms ON / 800ms OFF** | **Wi-Fi Connecting / SoftAP:** In setup portal or negotiating router. |
| **🌐 Rapid Double Blip** | **Web Traffic:** Incoming HTTP REST API or dashboard request. |
| **⚠️ 3 Pulses + Pause** | **Soil Moisture Sensor Fault:** Disconnected or voltage out of bounds. |
| **⚠️ 4 Pulses + Pause** | **Ultrasonic Sensor Fault:** Echo timeout, echo blocked, or disconnected. |
| **📉 Accelerando Frequency Sweep** | **Reservoir Empty:** Water level below dry run safety threshold. |
| **🔘 10 Hz Rapid Strobe (5s)** | **Hardware Factory Reset:** `BOOT` button (GPIO 9) held for 5 seconds. |

---

## 🏠 Home Assistant Integration

Integrate the plant monitor into Home Assistant using standard REST sensors:

```yaml
# configuration.yaml
sensor:
  - platform: rest
    name: "Plant Monitor"
    resource: "http://plants1.local/api/telemetry"
    scan_interval: 30
    json_attributes:
      - soil_moisture_pct
      - water_level_pct
      - water_level_cm
      - light_level_pct
      - dli
      - battery_voltage_mv
      - is_watering
      - wifi_rssi
    value_template: "{{ 'online' if value_json.is_watering is defined else 'offline' }}"

  - platform: template
    sensors:
      plant_soil_moisture:
        friendly_name: "Soil Moisture"
        unit_of_measurement: "%"
        value_template: "{{ state_attr('sensor.plant_monitor', 'soil_moisture_pct') | round(1) }}"
        icon_template: mdi:flower
      plant_water_level:
        friendly_name: "Water Level"
        unit_of_measurement: "%"
        value_template: "{{ state_attr('sensor.plant_monitor', 'water_level_pct') | round(1) }}"
        icon_template: mdi:water-percent
      plant_light_dli:
        friendly_name: "Daily Light Integral (DLI)"
        unit_of_measurement: "mol/m²/d"
        value_template: "{{ state_attr('sensor.plant_monitor', 'dli') | round(2) }}"
        icon_template: mdi:white-balance-sunny

# Manual Watering Trigger Action
rest_command:
  water_plant:
    url: "http://plants1.local/api/water"
    method: POST
    content_type: "application/json"
    payload: '{"duration_sec": 5}'
```

---

## 📂 Repository Structure

```
plant-esp32/
├── .github/workflows/          # CI/CD: Automated builds and release publishing
│   └── build-and-release.yml   # Multi-binary builder (factory + OTA .bin)
├── data/                       # Embedded Web Frontend (HTML, CSS, JS, PWA manifest)
├── installer/                  # ESP Web Tools 1-click in-browser web installer
├── firmware-rust/              # Rust Cargo Workspace
│   ├── Cargo.toml
│   ├── plant-core/             # Platform-independent domain logic, safety supervisor, math
│   └── plant-firmware/         # ESP-IDF firmware application, HAL, HTTP server, OTA client
├── pcb/                        # KiCad PCB project, schematics, and manufacturing Gerbers
├── enclosure/                  # 3D printable CAD models and enclosures
├── scripts/                    # Developer helper scripts (flashing, pin tests)
├── QUICKSTART.md               # First-time setup, flashing & provisioning guide
├── PROJECT.md                  # Comprehensive architectural system specification
└── CHANGELOG.md                # Firmware changelog history
```

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
