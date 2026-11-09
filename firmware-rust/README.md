# Plant ESP32-C3 Firmware (Rust Edition)

A modern, memory-safe, crash-resilient port of the Plant Monitor & Automated Irrigation Controller firmware for the ESP32-C3 SuperMini, written in Rust.

---

## Architecture

The Rust workspace is split into two independent crates:

```
firmware-rust/
├── Cargo.toml                # Cargo Workspace definition
├── README.md
├── plant-core/               # 100% Platform-Independent Logic & Domain Engine
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs            # Core exports
│       ├── config.rs         # Strongly-typed schema & partial patch updates
│       ├── reading.rs        # Sensor readings & formatting utilities
│       ├── ring_buffer.rs    # O(1) circular RAM buffer with FIFO/LIFO access
│       ├── safety.rs         # Safety supervisor & hardware-independent PumpGate trait
│       ├── sensors.rs        # Calibration, median filtering & metric arithmetic
│       ├── timeutil.rs       # NTP GMT offset & midnight quota rollover tracking
│       └── api.rs            # REST & WebSocket JSON serialization contract
└── plant-firmware/           # ESP32-C3 Target Firmware (ESP-IDF std)
    ├── Cargo.toml
    ├── build.rs
    ├── sdkconfig.defaults    # Low-power 80 MHz CPU & USB-CDC configuration
    ├── rust-toolchain.toml   # Nightly toolchain with build-std support
    ├── .cargo/config.toml    # riscv32imc-esp-espidf target configuration
    └── src/
        ├── main.rs           # Entry point & concurrent FreeRTOS/std tasks
        ├── pins.rs           # Hardware pin mapping matching KiCad schematic
        ├── pump.rs           # Low-side MOSFET driver & active-LOW status LED
        ├── sensors.rs        # ADC1 sampling, switched VCC duty-cycling, HC-SR04 timing
        ├── storage.rs        # Non-volatile NVS flash configuration storage
        ├── web.rs            # Embedded HTTP server & REST API
        ├── wifi.rs           # Wi-Fi STA driver with Modem Sleep enabled
        └── secrets.rs        # Local network Wi-Fi credentials
```

---

## Key Improvements over C++

1. **Host-First Testing (66/66 Unit Tests Passing):**
   - All domain logic (safety limits, cooldowns, daily quota, soil calibration, ultrasonic math, JSON serialization) runs and is tested directly on the host machine (`cargo test`).
2. **Deterministic Memory Safety & Zero Leaks:**
   - No raw pointers, buffer overflows, or heap fragmentation crashes.
3. **Hardware Invariant Guarantee:**
   - The pump cannot physically run unless all safety invariants (reservoir dry cutoff, cooldown timer, daily limits, night protection, and runtime watchdog) are satisfied simultaneously.
4. **Embedded Static Web Assets:**
   - Web assets (`index.html`, `app.js`, `style.css`) are embedded directly into the firmware flash binary via `include_str!`, eliminating filesystem corruption risks while keeping full REST API compatibility with the web interface.
5. **Low-Power Optimizations Preserved:**
   - ESP32-C3 CPU downclocked to 80 MHz (`CONFIG_ESP_DEFAULT_CPU_FREQ_MHZ_80=y`).
   - Wi-Fi Modem Sleep enabled (`WIFI_PS_MIN_MODEM`).
   - Switched probe power duty-cycling (GPIO 21 HIGH only during 30ms sampling) to eliminate galvanic corrosion and idle draw.
   - Ultrasonic stuck-latch recovery on GPIO 20 to heal clone HC-SR04 sensors.

---

## Hardware Pinout Reference (ESP32-C3 SuperMini)

Canonical Pinout Reference: [LastMinuteEngineers ESP32-C3 SuperMini Pinout](https://lastminuteengineers.com/esp32-c3-super-mini-pinout-reference/)

| Function | Pin | Hardware Circuit |
| :--- | :--- | :--- |
| **18650 Battery Mon** | GPIO 0 (ADC1_CH0) | 100k/100k Divider + 100nF C6 (from TB4-3) |
| **Ambient Light ADC** | GPIO 1 (ADC1_CH1) | GL55xx LDR + 10k Divider R7 |
| **Strapping NC**      | GPIO 2 | Unconnected / High-Z (Boot immune) |
| **Soil Moisture ADC** | GPIO 3 (ADC1_CH3) | Capacitive Probe Analog Out (TB2-1) |
| **5V Supply Monitor** | GPIO 4 (ADC1_CH4) | 100k/100k Divider + 100nF C1 |
| **I2C SCL**           | GPIO 5 | External Sensor Header (MOD1/MOD2) |
| **I2C SDA**           | GPIO 6 | External Sensor Header (MOD1/MOD2) |
| **Ultrasonic TRIG**   | GPIO 7 | Digital Out (10µs trigger pulse to TB1-2) |
| **Status LED**        | GPIO 8 | Digital Out (Active LOW onboard blue LED) |
| **Pump MOSFET Gate**  | GPIO 10 | Digital Out (AO3400A Low-side gate switch) |
| **Ultrasonic ECHO**   | GPIO 20 | Digital In via 1k/2k Divider from TB1-3 |
| **Soil Probe Power**  | GPIO 21 | Digital Out (Switched VCC to TB2-2) |

---

## How to Build & Flash

> [!TIP]
> Looking for the complete beginner's flashing guide? See the standalone **[QUICKSTART.md](../QUICKSTART.md)** for 1-click browser flashing and first-time Wi-Fi provisioning instructions.

> [!WARNING]
> **CRITICAL: NEVER FLASH WITHOUT THE DUAL-OTA PARTITION TABLE**
> The hardware MUST always be partitioned with `plant-firmware/partitions.csv` (`ota_0` and `ota_1`, 1984 KB each). 
> If `espflash` is run without `--partition-table`, it defaults to flashing a single 4 MB `factory` partition table, which permanently disables Over-The-Air (OTA) updates with error 258 (`ESP_ERR_INVALID_ARG`).

### Pre-Built Binaries (Zero Toolchain Installation)

Every GitHub release provides two distinct binary formats on the [Releases Page](https://github.com/thomaslr/plant-esp32/releases):

| Binary File | Flash Address | Purpose & Usage |
| :--- | :---: | :--- |
| **`plant-firmware-factory.bin`** | **`0x0`** | **Initial Setup / First-Time Flashing:** Merged factory binary containing the ESP32-C3 bootloader, Dual-OTA partition table (`ota_0` and `ota_1`, 1984 KB each), and application code. Flashes blank chips in one shot via Web Flasher or USB CLI. |
| **`plant-firmware.bin`** | OTA Slot | **Wi-Fi Updates (OTA):** Standalone application payload uploaded via the web interface (`http://plants1.local`) or downloaded during remote updates. |

#### 1. In-Browser Web Flasher (Recommended for Users)
1. Connect ESP32-C3 SuperMini via USB.
2. Put board in bootloader mode (Hold `BOOT` / GPIO 9 -> tap `RST` -> release `BOOT`).
3. Open [Espressif Web Flasher](https://espressif.github.io/esptool-js/) or [Adafruit WebSerial ESPTool](https://adafruit.github.io/Adafruit_WebSerial_ESPTool/).
4. Connect at `460800` baud.
5. Add `plant-firmware-factory.bin` at address **`0x0`**.
6. Click **Program / Flash**, then press `RST` to boot.

#### 2. Single-Command CLI (Pre-Built Binary)
```bash
# Using esptool.py
esptool.py --chip esp32c3 write_flash 0x0 plant-firmware-factory.bin

# Using espflash
espflash write-bin 0x0 plant-firmware-factory.bin
```

---

### Developing & Flashing from Source

#### 1. Run Host Unit Tests
```bash
cargo test --manifest-path plant-core/Cargo.toml --target aarch64-apple-darwin
```

#### 2. Configure Wi-Fi (Optional - or use Captive Portal)
Copy `plant-firmware/src/secrets.rs.example` to `plant-firmware/src/secrets.rs` and update your SSID and password (or leave blank to use the onboard SoftAP captive portal).

#### 3. Flash to ESP32-C3 SuperMini (USB-C)
Plug your ESP32-C3 into USB, then run:
```bash
# Recommended: Uses .cargo/config.toml runner which automatically includes the partition table
cargo run --release

# Or use the flash script:
../scripts/flash_firmware.sh

# If invoking espflash directly, ALWAYS pass the partition table:
espflash flash --partition-table plant-firmware/partitions.csv target/riscv32imc-esp-espidf/release/plant-firmware --monitor
```

#### 4. Packaging Release Binaries Locally
```bash
# Generate OTA application payload
espflash save-image --chip esp32c3 \
  --partition-table plant-firmware/partitions.csv \
  target/riscv32imc-esp-espidf/release/plant-firmware \
  plant-firmware.bin

# Generate 0x0 merged factory binary (Bootloader + Partitions + App)
espflash save-image --merge --chip esp32c3 \
  --partition-table plant-firmware/partitions.csv \
  target/riscv32imc-esp-espidf/release/plant-firmware \
  plant-firmware-factory.bin
```
