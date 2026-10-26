//! Hardware-independent core of the Plant ESP32 controller.
//!
//! This crate is a Rust port of the portable parts of the original C++/Arduino
//! firmware (`include/*.h`, `src/*.cpp`):
//!
//! | C++ file                             | Rust module                  |
//! |--------------------------------------|------------------------------|
//! | `config_schema.h`/`config_manager.h` | [`config`]                   |
//! | `ring_buffer.h`                      | [`ring_buffer`], [`reading`] |
//! | `safety_manager.{h,cpp}`             | [`safety`]                   |
//! | maths in `sensor_manager.cpp`        | [`sensors`]                  |
//! | NTP/midnight helpers in `main.cpp`   | [`timeutil`]                 |
//! | JSON in `web_server.cpp`             | [`api`]                      |
//!
//! Nothing here touches GPIO, ADC, Wi-Fi or the filesystem. The firmware crate
//! supplies those through small traits (e.g. [`safety::PumpGate`]) and passes a
//! monotonic millisecond clock in explicitly. That makes every safety rule
//! testable on the host with a fake clock and a fake pump.

pub mod api;
pub mod config;
pub mod reading;
pub mod ring_buffer;
pub mod safety;
pub mod sensors;
pub mod timeutil;
