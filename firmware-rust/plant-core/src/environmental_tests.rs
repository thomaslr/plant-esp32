use crate::config::AppConfig;
use crate::reading::SensorReading;

#[test]
fn test_environmental_config_defaults() {
    let cfg = AppConfig::default();
    assert!(!cfg.gauges.environmental.sht30_enabled);
    assert!(!cfg.gauges.environmental.bh1750_enabled);
    assert!(!cfg.gauges.environmental.ldr_enabled);
}

#[test]
fn test_environmental_config_patch() {
    let mut cfg = AppConfig::default();
    let patch: crate::config::ConfigPatch = serde_json::from_str(
        r#"{"gauges":{"environmental":{"sht30_enabled":true,"bh1750_enabled":true,"ldr_enabled":true}}}"#
    ).unwrap();
    cfg.apply_patch(&patch).unwrap();
    assert!(cfg.gauges.environmental.sht30_enabled);
    assert!(cfg.gauges.environmental.bh1750_enabled);
    assert!(cfg.gauges.environmental.ldr_enabled);
}

#[test]
fn test_environmental_reading_serialisation() {
    let mut r = SensorReading::default();
    r.air_temp_c = Some(26.42);
    r.humidity_pct = Some(61.88);
    r.light_lux = Some(320.5);
    r.ldr_percent = Some(72.1);

    let log = crate::api::LogEntry::from(&r);
    assert_eq!(log.air_temp_c, Some(26.4));
    assert_eq!(log.humidity_pct, Some(61.9));
    assert_eq!(log.light_lux, Some(320.5));
    assert_eq!(log.ldr_percent, Some(72.1));

    let json = serde_json::to_string(&log).unwrap();
    assert!(json.contains("\"air_temp_c\":26.4"));
    assert!(json.contains("\"humidity_pct\":61.9"));
    assert!(json.contains("\"light_lux\":320.5"));
    assert!(json.contains("\"ldr_percent\":72.1"));
}
