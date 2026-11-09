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

/// Compares two semver strings (e.g. "v0.2.35" vs "0.2.34", "v0.3.0" vs "v0.2.34").
/// Returns true if `candidate` is strictly greater (newer) than `reference`.
pub fn is_version_newer(candidate: &str, reference: &str) -> bool {
    let parse = |s: &str| -> (u32, u32, u32) {
        let clean = s.trim().trim_start_matches('v').trim_start_matches('V');
        let mut parts = clean.split('.').filter_map(|p| p.parse::<u32>().ok());
        let major = parts.next().unwrap_or(0);
        let minor = parts.next().unwrap_or(0);
        let patch = parts.next().unwrap_or(0);
        (major, minor, patch)
    };
    parse(candidate) > parse(reference)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_version_newer("v0.2.35", "v0.2.34"));
        assert!(is_version_newer("v0.2.35", "0.2.34"));
        assert!(is_version_newer("0.3.0", "v0.2.34"));
        assert!(is_version_newer("v1.0.0", "v0.9.9"));
        assert!(!is_version_newer("v0.2.34", "v0.2.34"));
        assert!(!is_version_newer("0.2.34", "0.2.34"));
        assert!(!is_version_newer("v0.2.33", "v0.2.34"));
        assert!(!is_version_newer("", "v0.2.34"));
    }
}

#[cfg(test)]
mod environmental_tests;

#[cfg(test)]
mod i2c_tests;
