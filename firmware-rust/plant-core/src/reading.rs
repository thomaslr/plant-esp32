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

impl SensorReading {
    pub const BINARY_SIZE: usize = 57;

    /// Serializes reading into fixed 57-byte binary layout.
    pub fn to_binary(&self) -> [u8; Self::BINARY_SIZE] {
        let mut buf = [0u8; Self::BINARY_SIZE];
        buf[0..4].copy_from_slice(&self.timestamp.to_le_bytes());
        buf[4] = if self.pump_active { 1 } else { 0 };
        buf[5..9].copy_from_slice(&self.supply_voltage.to_le_bytes());
        buf[9..13].copy_from_slice(&self.battery_voltage.to_le_bytes());
        buf[13..17].copy_from_slice(&self.chip_temp.to_le_bytes());
        buf[17..21].copy_from_slice(&self.soil_raw_adc.to_le_bytes());
        buf[21..25].copy_from_slice(&self.soil_percent.to_le_bytes());
        buf[25..29].copy_from_slice(&self.distance_cm.to_le_bytes());
        buf[29..33].copy_from_slice(&self.water_level_cm.to_le_bytes());
        buf[33..37].copy_from_slice(&self.water_percent.to_le_bytes());
        buf[37..41].copy_from_slice(&self.air_temp_c.unwrap_or(f32::NAN).to_le_bytes());
        buf[41..45].copy_from_slice(&self.humidity_pct.unwrap_or(f32::NAN).to_le_bytes());
        buf[45..49].copy_from_slice(&self.light_lux.unwrap_or(f32::NAN).to_le_bytes());
        buf[49..53].copy_from_slice(&self.ldr_percent.unwrap_or(f32::NAN).to_le_bytes());
        // Remaining 4 bytes reserved for future sensors
        buf
    }

    /// Deserializes reading from fixed 57-byte binary layout.
    pub fn from_binary(buf: &[u8]) -> Option<Self> {
        if buf.len() < Self::BINARY_SIZE {
            return None;
        }
        let timestamp = u32::from_le_bytes(buf[0..4].try_into().ok()?);
        let pump_active = buf[4] != 0;
        let supply_voltage = f32::from_le_bytes(buf[5..9].try_into().ok()?);
        let battery_voltage = f32::from_le_bytes(buf[9..13].try_into().ok()?);
        let chip_temp = f32::from_le_bytes(buf[13..17].try_into().ok()?);
        let soil_raw_adc = i32::from_le_bytes(buf[17..21].try_into().ok()?);
        let soil_percent = f32::from_le_bytes(buf[21..25].try_into().ok()?);
        let distance_cm = f32::from_le_bytes(buf[25..29].try_into().ok()?);
        let water_level_cm = f32::from_le_bytes(buf[29..33].try_into().ok()?);
        let water_percent = f32::from_le_bytes(buf[33..37].try_into().ok()?);

        let a_t = f32::from_le_bytes(buf[37..41].try_into().ok()?);
        let air_temp_c = if a_t.is_nan() { None } else { Some(a_t) };

        let hum = f32::from_le_bytes(buf[41..45].try_into().ok()?);
        let humidity_pct = if hum.is_nan() { None } else { Some(hum) };

        let lux = f32::from_le_bytes(buf[45..49].try_into().ok()?);
        let light_lux = if lux.is_nan() { None } else { Some(lux) };

        let ldr = f32::from_le_bytes(buf[49..53].try_into().ok()?);
        let ldr_percent = if ldr.is_nan() { None } else { Some(ldr) };

        Some(Self {
            timestamp,
            soil_percent,
            soil_raw_adc,
            water_level_cm,
            water_percent,
            distance_cm,
            supply_voltage,
            battery_voltage,
            chip_temp,
            pump_active,
            air_temp_c,
            humidity_pct,
            light_lux,
            ldr_percent,
        })
    }
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

    #[test]
    fn binary_serialization_roundtrip() {
        let r = SensorReading {
            timestamp: 1794156308,
            soil_percent: 69.9,
            soil_raw_adc: 1055,
            water_level_cm: 11.3,
            water_percent: 40.3,
            distance_cm: 18.7,
            supply_voltage: 5.14,
            battery_voltage: 3.85,
            chip_temp: 38.7,
            pump_active: true,
            air_temp_c: Some(28.4),
            humidity_pct: Some(65.2),
            light_lux: Some(420.5),
            ldr_percent: None,
        };

        let bin = r.to_binary();
        assert_eq!(bin.len(), SensorReading::BINARY_SIZE);
        let decoded = SensorReading::from_binary(&bin).expect("decode failed");
        assert_eq!(decoded.timestamp, r.timestamp);
        assert_eq!(decoded.pump_active, r.pump_active);
        assert_eq!(decoded.soil_percent, r.soil_percent);
        assert_eq!(decoded.soil_raw_adc, r.soil_raw_adc);
        assert_eq!(decoded.water_level_cm, r.water_level_cm);
        assert_eq!(decoded.air_temp_c, r.air_temp_c);
        assert_eq!(decoded.humidity_pct, r.humidity_pct);
        assert_eq!(decoded.light_lux, r.light_lux);
        assert_eq!(decoded.ldr_percent, None);
    }
}
