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
   - Switched probe power duty-cycling (GPIO 5 HIGH only during 30ms sampling) to eliminate galvanic corrosion and idle draw.
   - Ultrasonic stuck-latch recovery on GPIO 6 to heal clone HC-SR04 sensors.

---

## Hardware Pinout Reference (ESP32-C3 SuperMini)

| Function | Pin | Hardware Circuit |
| :--- | :--- | :--- |
| **5V Supply Monitor** | GPIO 0 (ADC1_CH0) | 100k/100k Divider + 100nF C1 |
| **Soil Moisture ADC** | GPIO 1 (ADC1_CH1) | Capacitive Probe Analog Out |
| **Ambient Light ADC**| GPIO 3 (ADC1_CH3) | GL55xx LDR + 10k Divider |
| **18650 Battery Mon** | GPIO 4 (ADC1_CH4) | 100k/100k Divider + 100nF C6 |
| **Soil Probe Power**  | GPIO 5 | Digital Out (Switched VCC) |
| **Ultrasonic ECHO**   | GPIO 6 | Digital In via 1k/2k Divider |
| **Pump MOSFET Gate**  | GPIO 7 | Digital Out (Low-side gate switch) |
| **Status LED**        | GPIO 8 | Digital Out (Active LOW) |
| **Ultrasonic TRIG**   | GPIO 10 | Digital Out (10µs trigger pulse) |
| **I2C SDA**           | GPIO 20 | External Sensor Header |
| **I2C SCL**           | GPIO 21 | External Sensor Header |

---

## How to Build & Flash

### 1. Run Host Unit Tests
```bash
cargo test --manifest-path plant-core/Cargo.toml
```

### 2. Configure Wi-Fi
Copy `plant-firmware/src/secrets.rs.example` to `plant-firmware/src/secrets.rs` and update your SSID and password.

### 3. Flash to ESP32-C3 SuperMini
Plug your ESP32-C3 into USB, then run:
```bash
cd plant-firmware
cargo espflash flash --monitor
```
