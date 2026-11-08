//! Sensor sample type (from `ring_buffer.h`) and the display rounding helpers
//! used throughout the JSON API.

/// One complete acquisition cycle.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SensorReading {
    /// Seconds since boot.
    pub timestamp: u32,
    /// Calculated soil moisture percentage.
    pub soil_percent: f32,
    /// Calibrated soil sensor millivolts.
    pub soil_raw_adc: i32,
    /// Calculated water depth in the tank.
    pub water_level_cm: f32,
    /// Tank fullness percentage.
    pub water_percent: f32,
    /// Raw ultrasonic distance to the water surface (`-1.0` = sensor fault).
    pub distance_cm: f32,
    /// Measured 5 V rail voltage.
    pub supply_voltage: f32,
    /// Measured 18650 battery voltage.
    pub battery_voltage: f32,
    /// Internal silicon temperature.
    pub chip_temp: f32,
    /// True if the pump was running during the sample.
    pub pump_active: bool,
    /// Ambient air temperature in °C from SHT30 (`None` if disabled or sensor missing).
    pub air_temp_c: Option<f32>,
    /// Relative humidity percentage from SHT30 (`None` if disabled or sensor missing).
    pub humidity_pct: Option<f32>,
    /// Ambient light level in Lux from BH1750 (`None` if disabled or sensor missing).
    pub light_lux: Option<f32>,
    /// Ambient light percentage (0-100%) from analog LDR (`None` if disabled).
    pub ldr_percent: Option<f32>,
}

/// `roundf(x * 10) / 10` - one decimal place, half away from zero, in `f32`
/// exactly like the C++ code so JSON output is identical.
pub fn round1(x: f32) -> f32 {
    (x * 10.0).round() / 10.0
}

/// `roundf(x * 100) / 100` - two decimal places.
pub fn round2(x: f32) -> f32 {
    (x * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounding_matches_roundf_semantics() {
        assert_eq!(round1(23.44), 23.4);
        assert_eq!(round1(23.46), 23.5);
        assert_eq!(round1(-0.04), 0.0);
        assert_eq!(round2(4.987), 4.99);
        assert_eq!(round2(3.304), 3.3);
    }
}
