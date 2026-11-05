//! Pure sensor maths extracted from `sensor_manager.cpp`.
//!
//! The actual GPIO/ADC access lives in the firmware crate; everything that is
//! just arithmetic (median filtering, calibration curves, tank geometry) is
//! here so it can be unit-tested without hardware.

/// HC-SR04 round-trip time to distance: sound travels ~58.2 us per cm (there and back).
pub const US_PER_CM: f32 = 58.2;
/// HC-SR04 valid range.
pub const MIN_VALID_DISTANCE_CM: f32 = 2.0;
pub const MAX_VALID_DISTANCE_CM: f32 = 400.0;
/// Both supply monitors use a 100 k / 100 k divider, so the real voltage is 2x the pin voltage.
pub const DIVIDER_RATIO: f32 = 2.0;

/// Median of `values` (sorted in place). Even-length input averages the two middle samples.
pub fn median_f32(values: &mut [f32]) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(|a, b| a.total_cmp(b));
    let n = values.len();
    Some(if n % 2 == 0 { (values[n / 2 - 1] + values[n / 2]) / 2.0 } else { values[n / 2] })
}

/// Integer median; even-length input uses integer division like the C++ `calculateMedianInt`.
pub fn median_i32(values: &mut [i32]) -> Option<i32> {
    if values.is_empty() {
        return None;
    }
    values.sort_unstable();
    let n = values.len();
    Some(if n % 2 == 0 { (values[n / 2 - 1] + values[n / 2]) / 2 } else { values[n / 2] })
}

/// Converts an ECHO pulse width to centimetres.
pub fn echo_us_to_cm(duration_us: i64) -> f32 {
    duration_us as f32 / US_PER_CM
}

/// True if the distance is inside the HC-SR04's specified range.
pub fn is_valid_distance(cm: f32) -> bool {
    (MIN_VALID_DISTANCE_CM..=MAX_VALID_DISTANCE_CM).contains(&cm)
}

/// Plausible operational range for capacitive soil moisture sensor raw ADC reading (in mV).
///
/// Typical readings:
/// - Bone dry air: ~2600 - 2900 mV
/// - Fully submerged in water: ~1000 - 1300 mV
/// - Disconnected probe / broken trace / dead 555 timer: < 300 mV
/// - Short circuit to 3.3V rail: > 3200 mV
pub const MIN_VALID_SOIL_RAW_MV: i32 = 300;
pub const MAX_VALID_SOIL_RAW_MV: i32 = 3200;

/// Returns true if the raw soil probe ADC voltage is within plausible operational bounds.
pub fn is_valid_soil_raw(raw_mv: i32) -> bool {
    (MIN_VALID_SOIL_RAW_MV..=MAX_VALID_SOIL_RAW_MV).contains(&raw_mv)
}

/// Capacitive soil probes have an *inverted* curve: high millivolts in dry
/// air, low when submerged. Returns 0-100 %. `dry == wet` returns 0.
pub fn soil_percent(raw_mv: i32, dry_mv: i32, wet_mv: i32) -> f32 {
    if dry_mv == wet_mv {
        return 0.0;
    }
    let pct = 100.0 * (1.0 - (raw_mv - wet_mv) as f32 / (dry_mv - wet_mv) as f32);
    pct.clamp(0.0, 100.0)
}

/// Millivolts at a 1:1 divider centre tap -> real rail/battery volts.
pub fn divider_volts(median_mv: i32) -> f32 {
    (median_mv as f32 * DIVIDER_RATIO) / 1000.0
}

/// Tank geometry derived from one ultrasonic distance reading.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaterMetrics {
    /// Water depth in the tank (0 if the sensor failed).
    pub level_cm: f32,
    /// Fullness over the usable span (0-100).
    pub percent: f32,
}

/// `distance_cm <= 0` (sensor fault, `-1.0`) yields an **empty** tank so every
/// downstream safety check fails safe.
pub fn water_metrics(distance_cm: f32, tank_height_cm: f32, sensor_offset_cm: f32) -> WaterMetrics {
    // `!(x > 0.0)` also catches NaN.
    if !(distance_cm > 0.0) {
        return WaterMetrics { level_cm: 0.0, percent: 0.0 };
    }
    let level = (tank_height_cm - distance_cm).clamp(0.0, tank_height_cm.max(0.0));
    let usable_span = tank_height_cm - sensor_offset_cm;
    let percent =
        if usable_span > 0.0 { ((level / usable_span) * 100.0).clamp(0.0, 100.0) } else { 0.0 };
    WaterMetrics { level_cm: level, percent }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_odd_even_and_empty() {
        assert_eq!(median_f32(&mut [3.0, 1.0, 2.0]), Some(2.0));
        assert_eq!(median_f32(&mut [4.0, 1.0, 3.0, 2.0]), Some(2.5));
        assert_eq!(median_f32(&mut [7.5]), Some(7.5));
        assert_eq!(median_f32(&mut []), None);
        assert_eq!(median_i32(&mut [900, 100, 500, 300, 700]), Some(500));
        assert_eq!(median_i32(&mut [10, 20]), Some(15));
        assert_eq!(median_i32(&mut [10, 21]), Some(15)); // integer division, like C++
        assert_eq!(median_i32(&mut []), None);
    }

    #[test]
    fn median_rejects_single_outlier() {
        // A spike from ADC noise must not move the result.
        assert_eq!(median_i32(&mut [1500, 1498, 3300, 1502, 1499]), Some(1500));
    }

    #[test]
    fn echo_conversion() {
        assert!((echo_us_to_cm(582) - 10.0).abs() < 1e-4);
        assert!((echo_us_to_cm(5820) - 100.0).abs() < 1e-3);
    }

    #[test]
    fn distance_validity_range() {
        assert!(!is_valid_distance(1.99));
        assert!(is_valid_distance(2.0));
        assert!(is_valid_distance(400.0));
        assert!(!is_valid_distance(400.01));
        assert!(!is_valid_distance(-1.0));
        assert!(!is_valid_distance(f32::NAN));
    }

    #[test]
    fn soil_raw_validity_range() {
        assert!(!is_valid_soil_raw(0));
        assert!(!is_valid_soil_raw(150));
        assert!(!is_valid_soil_raw(299));
        assert!(is_valid_soil_raw(300));
        assert!(is_valid_soil_raw(1100)); // typical wet
        assert!(is_valid_soil_raw(2800)); // typical dry
        assert!(is_valid_soil_raw(3200));
        assert!(!is_valid_soil_raw(3201));
        assert!(!is_valid_soil_raw(3300));
    }

    #[test]
    fn soil_percent_endpoints_and_inversion() {
        // defaults: dry 2800 mV, wet 1100 mV
        assert_eq!(soil_percent(2800, 2800, 1100), 0.0);
        assert_eq!(soil_percent(1100, 2800, 1100), 100.0);
        assert!((soil_percent(1950, 2800, 1100) - 50.0).abs() < 1e-3);
        // wetter soil -> lower voltage -> higher percent
        assert!(soil_percent(1500, 2800, 1100) > soil_percent(2000, 2800, 1100));
    }

    #[test]
    fn soil_percent_clamps_out_of_range() {
        assert_eq!(soil_percent(3300, 2800, 1100), 0.0);
        assert_eq!(soil_percent(500, 2800, 1100), 100.0);
    }

    #[test]
    fn soil_percent_degenerate_calibration() {
        assert_eq!(soil_percent(1000, 2000, 2000), 0.0);
    }

    #[test]
    fn divider_maths() {
        assert!((divider_volts(2500) - 5.0).abs() < 1e-6);
        assert!((divider_volts(1850) - 3.7).abs() < 1e-6);
        assert_eq!(divider_volts(0), 0.0);
    }

    #[test]
    fn water_metrics_normal() {
        // 30 cm tank, 3 cm offset. Surface 10 cm below sensor -> 20 cm of water.
        let w = water_metrics(10.0, 30.0, 3.0);
        assert!((w.level_cm - 20.0).abs() < 1e-5);
        assert!((w.percent - (20.0 / 27.0 * 100.0)).abs() < 1e-3);
    }

    #[test]
    fn water_metrics_full_and_empty_clamps() {
        assert_eq!(water_metrics(2.0, 30.0, 3.0).level_cm, 28.0);
        assert_eq!(water_metrics(2.0, 30.0, 3.0).percent, 100.0); // 28/27 clamps
        let empty = water_metrics(45.0, 30.0, 3.0);
        assert_eq!(empty.level_cm, 0.0);
        assert_eq!(empty.percent, 0.0);
    }

    #[test]
    fn sensor_fault_reports_empty_tank_so_safety_fails_safe() {
        for bad in [-1.0, 0.0, f32::NAN] {
            let w = water_metrics(bad, 30.0, 3.0);
            assert_eq!(w, WaterMetrics { level_cm: 0.0, percent: 0.0 }, "distance {bad}");
        }
    }

    #[test]
    fn degenerate_geometry_does_not_divide_by_zero() {
        assert_eq!(water_metrics(10.0, 3.0, 3.0).percent, 0.0);
        assert_eq!(water_metrics(10.0, 2.0, 3.0).percent, 0.0);
    }
}
