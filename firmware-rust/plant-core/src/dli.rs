//! Daily Light Integral (DLI) calculation in mol/m²/day for plant horticulture.
//!
//! DLI measures the total amount of Photosynthetically Active Radiation (PAR)
//! delivered to a plant over a 24-hour day.
//!
//! Standard daylight / solar conversion:
//! 1 lux ≈ 0.0185 μmol/m²/s PPFD (Photosynthetic Photon Flux Density).
//! DLI = ∫ PPFD dt / 1,000,000 (mol/m²/day).

use crate::reading::{round2, SensorReading};
use crate::timeutil::{local_day, MIN_VALID_UNIX_SECS};

/// Sunlight / daylight conversion factor: 1 lux ≈ 0.0185 μmol/m²/s PPFD.
pub const LUX_TO_PPFD_SUNLIGHT: f32 = 0.0185;

/// Categorical Horticultural Band classifications for instantaneous lux.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorticulturalBand {
    /// 0 - 250 lx: Insufficient for vegetative growth; maintenance/dormant only.
    DeepShade,
    /// 250 - 1,000 lx: Low light tolerant houseplants (Snake plant, ZZ plant, Cast iron).
    LowLight,
    /// 1,000 - 2,500 lx: Medium/filtered light (Pothos, Monstera, Philodendron, Ferns).
    MediumLight,
    /// 2,500 - 5,000 lx: Bright indirect light (Ficus, Orchids, indoor Herbs, Begonias).
    BrightIndirect,
    /// > 5,000 lx: Direct sunlight / high light (Succulents, Cacti, outdoor vegetables).
    DirectSun,
}

impl HorticulturalBand {
    /// Classifies an instantaneous illuminance reading in lux into its horticultural band.
    pub fn from_lux(lux: f32) -> Self {
        if lux < 250.0 {
            Self::DeepShade
        } else if lux < 1000.0 {
            Self::LowLight
        } else if lux < 2500.0 {
            Self::MediumLight
        } else if lux < 5000.0 {
            Self::BrightIndirect
        } else {
            Self::DirectSun
        }
    }

    /// Returns human-readable label for the band.
    pub fn label(&self) -> &'static str {
        match self {
            Self::DeepShade => "Deep Shade",
            Self::LowLight => "Low Light",
            Self::MediumLight => "Medium Light",
            Self::BrightIndirect => "Bright Indirect",
            Self::DirectSun => "Direct Sun",
        }
    }

    /// Returns typical plants thriving in this band.
    pub fn description(&self) -> &'static str {
        match self {
            Self::DeepShade => "Maintenance only (<250 lx)",
            Self::LowLight => "Snake plant, ZZ (250–1k lx)",
            Self::MediumLight => "Pothos, Monstera (1k–2.5k lx)",
            Self::BrightIndirect => "Ficus, Orchids (2.5k–5k lx)",
            Self::DirectSun => "Cacti, Succulents (>5k lx)",
        }
    }
}

/// DLI quality ratings in mol/m²/day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DliRating {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

impl DliRating {
    pub fn from_dli(dli: f32) -> Self {
        if dli < 2.0 {
            Self::VeryLow
        } else if dli < 5.0 {
            Self::Low
        } else if dli < 10.0 {
            Self::Medium
        } else if dli < 16.0 {
            Self::High
        } else {
            Self::VeryHigh
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::VeryLow => "Very Low (<2 mol)",
            Self::Low => "Low (2–5 mol)",
            Self::Medium => "Medium (5–10 mol)",
            Self::High => "High (10–16 mol)",
            Self::VeryHigh => "Full Sun (>16 mol)",
        }
    }
}

/// Computes the Daily Light Integral (DLI) in mol/m²/day for a specific local calendar day
/// using trapezoidal numerical integration across chronological samples.
///
/// Returns `None` if no valid light samples exist for the requested day.
pub fn calculate_daily_dli(
    readings: &[SensorReading],
    target_day: i64,
    gmt_offset_hours: i32,
) -> Option<f32> {
    // Extract (timestamp_secs, lux) for the target local day
    let mut day_samples: Vec<(u32, f32)> = readings
        .iter()
        .filter_map(|r| {
            if (r.timestamp as i64) < MIN_VALID_UNIX_SECS {
                return None;
            }
            if local_day(r.timestamp as i64, gmt_offset_hours) != Some(target_day) {
                return None;
            }
            r.light_lux.map(|lux| (r.timestamp, lux))
        })
        .collect();

    if day_samples.is_empty() {
        return None;
    }

    // Ensure chronological sorting
    day_samples.sort_by_key(|&(ts, _)| ts);

    if day_samples.len() == 1 {
        // Single sample: assume it represents a nominal 1-hour interval (3600s)
        let (_, lux) = day_samples[0];
        let ppfd = lux * LUX_TO_PPFD_SUNLIGHT;
        let micro_moles = ppfd * 3600.0;
        return Some(round2(micro_moles / 1_000_000.0));
    }

    let mut total_micro_moles = 0.0f32;

    for window in day_samples.windows(2) {
        let (t0, lux0) = window[0];
        let (t1, lux1) = window[1];

        if t1 <= t0 {
            continue;
        }

        // Cap delta at 2 hours (7200s) to avoid unphysical integration across huge sample gaps
        let delta_sec = (t1 - t0).min(7200) as f32;
        let avg_lux = (lux0 + lux1) / 2.0;
        let ppfd = avg_lux * LUX_TO_PPFD_SUNLIGHT;
        total_micro_moles += ppfd * delta_sec;
    }

    let dli = total_micro_moles / 1_000_000.0;
    Some(round2(dli))
}

#[cfg(test)]
mod tests {
    use super::*;

    const T0: i64 = 1_741_564_800; // 2025-03-10T00:00:00Z

    #[test]
    fn test_horticultural_bands() {
        assert_eq!(HorticulturalBand::from_lux(0.0), HorticulturalBand::DeepShade);
        assert_eq!(HorticulturalBand::from_lux(108.0), HorticulturalBand::DeepShade);
        assert_eq!(HorticulturalBand::from_lux(250.0), HorticulturalBand::LowLight);
        assert_eq!(HorticulturalBand::from_lux(600.0), HorticulturalBand::LowLight);
        assert_eq!(HorticulturalBand::from_lux(1000.0), HorticulturalBand::MediumLight);
        assert_eq!(HorticulturalBand::from_lux(1800.0), HorticulturalBand::MediumLight);
        assert_eq!(HorticulturalBand::from_lux(2500.0), HorticulturalBand::BrightIndirect);
        assert_eq!(HorticulturalBand::from_lux(3500.0), HorticulturalBand::BrightIndirect);
        assert_eq!(HorticulturalBand::from_lux(5000.0), HorticulturalBand::DirectSun);
        assert_eq!(HorticulturalBand::from_lux(10000.0), HorticulturalBand::DirectSun);
    }

    #[test]
    fn test_dli_calculation_constant_light() {
        // 12 hours of 1,000 lux (e.g. 1 sample every hour from 06:00 to 18:00 local)
        let day = local_day(T0, 0).unwrap();
        let mut readings = Vec::new();

        for hour in 6..=18 {
            let ts = (T0 + hour * 3600) as u32;
            let mut r = SensorReading::default();
            r.timestamp = ts;
            r.light_lux = Some(1000.0);
            readings.push(r);
        }

        let dli = calculate_daily_dli(&readings, day, 0).expect("Should compute DLI");
        // 1000 lux * 0.0185 = 18.5 μmol/m²/s
        // 12 hours = 43,200 s
        // Total = 18.5 * 43,200 = 799,200 μmol ≈ 0.80 mol/m²/day
        assert!((dli - 0.80).abs() <= 0.05, "Expected ~0.80 mol, got {}", dli);
    }

    #[test]
    fn test_dli_calculation_ignores_other_days() {
        let day1 = local_day(T0, 0).unwrap();
        let day2 = day1 + 1;

        let mut readings = Vec::new();

        // Day 1: 1000 lux for 10 hours
        for hour in 8..=18 {
            let ts = (T0 + hour * 3600) as u32;
            let mut r = SensorReading::default();
            r.timestamp = ts;
            r.light_lux = Some(1000.0);
            readings.push(r);
        }

        // Day 2: 5000 lux for 10 hours
        for hour in 8..=18 {
            let ts = (T0 + 86400 + hour * 3600) as u32;
            let mut r = SensorReading::default();
            r.timestamp = ts;
            r.light_lux = Some(5000.0);
            readings.push(r);
        }

        let dli_day1 = calculate_daily_dli(&readings, day1, 0).unwrap();
        let dli_day2 = calculate_daily_dli(&readings, day2, 0).unwrap();

        assert!(dli_day2 > dli_day1 * 4.5);
    }

    #[test]
    fn test_dli_ratings() {
        assert_eq!(DliRating::from_dli(1.2), DliRating::VeryLow);
        assert_eq!(DliRating::from_dli(3.5), DliRating::Low);
        assert_eq!(DliRating::from_dli(7.5), DliRating::Medium);
        assert_eq!(DliRating::from_dli(12.0), DliRating::High);
        assert_eq!(DliRating::from_dli(18.0), DliRating::VeryHigh);
    }
}
