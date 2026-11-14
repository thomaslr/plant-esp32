//! Non-Volatile Flash Configuration & Wi-Fi Storage using ESP32 NVS.
//!
//! Stores `AppConfig` under the `plant_cfg` namespace and
//! network credentials under the `plant_wifi` namespace.

use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};
use log::{debug, error, info, warn};
use plant_core::config::AppConfig;
use plant_core::reading::{SensorReading, MAX_HISTORY_BLOB_BYTES, MAX_HISTORY_CAPACITY};
use plant_core::ring_buffer::RingBuffer;

const CFG_NAMESPACE: &str = "plant_cfg";
const NVS_KEY_CONFIG: &str = "config_json";

const WIFI_NAMESPACE: &str = "plant_wifi";
const NVS_KEY_SSID: &str = "wifi_ssid";
const NVS_KEY_PASS: &str = "wifi_pass";

const LOG_NAMESPACE: &str = "plant_log";
const NVS_KEY_HISTORY: &str = "hist_bin";

const NVS_KEY_SOIL_DRY: &str = "soil_dry";
const NVS_KEY_SOIL_WET: &str = "soil_wet";

const BUFFER_SIZE: usize = 4096;

/// Stored network credentials for Station mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiCredentials {
    pub ssid: String,
    pub password: String,
}

pub struct NvsConfigStorage {
    nvs_cfg: EspNvs<NvsDefault>,
    nvs_wifi: EspNvs<NvsDefault>,
    nvs_log: EspNvs<NvsDefault>,
}

impl NvsConfigStorage {
    /// Opens the NVS partitions for configuration and Wi-Fi credentials.
    pub fn new(partition: EspDefaultNvsPartition) -> Result<Self, esp_idf_svc::sys::EspError> {
        let nvs_cfg = EspNvs::new(partition.clone(), CFG_NAMESPACE, true)?;
        let nvs_wifi = EspNvs::new(partition.clone(), WIFI_NAMESPACE, true)?;
        let nvs_log = EspNvs::new(partition, LOG_NAMESPACE, true)?;
        Ok(Self { nvs_cfg, nvs_wifi, nvs_log })
    }

    /// Loads the stored `AppConfig`. Falls back to default if not found or invalid.
    pub fn load(&self) -> AppConfig {
        let mut buf = vec![0u8; BUFFER_SIZE];
        let mut config = match self.nvs_cfg.get_str(NVS_KEY_CONFIG, &mut buf) {
            Ok(Some(json_str)) => match serde_json::from_str::<AppConfig>(json_str) {
                Ok(cfg) => {
                    info!("[NVS] Loaded existing configuration from flash");
                    cfg
                }
                Err(e) => {
                    warn!("[NVS] Config JSON parse warning ({}). Attempting patch recovery...", e);
                    if let Ok(patch) = serde_json::from_str::<plant_core::config::ConfigPatch>(json_str) {
                        let mut recovered = AppConfig::default();
                        if let Ok(()) = recovered.apply_patch(&patch) {
                            info!("[NVS] Successfully recovered configuration from stored patch");
                            recovered
                        } else {
                            warn!("[NVS] Patch validation failed; using in-memory defaults without overwriting flash.");
                            AppConfig::default()
                        }
                    } else {
                        warn!("[NVS] Corrupt config JSON in NVS; using in-memory defaults without overwriting flash.");
                        AppConfig::default()
                    }
                }
            },
            Ok(None) => {
                info!("[NVS] No configuration found in flash. Writing initial defaults.");
                let def = AppConfig::default();
                let _ = self.save(&def);
                def
            }
            Err(e) => {
                error!("[NVS] Failed to read from NVS ({}). Using defaults.", e);
                AppConfig::default()
            }
        };

        // Cross-check direct NVS hardware calibration keys
        match self.nvs_cfg.get_i32(NVS_KEY_SOIL_DRY) {
            Ok(Some(dry)) if dry > 0 => {
                info!("[NVS] Overriding soil dry_adc from dedicated NVS key: {} mV", dry);
                config.gauges.soil_moisture.dry_adc = dry;
            }
            _ => {}
        }
        match self.nvs_cfg.get_i32(NVS_KEY_SOIL_WET) {
            Ok(Some(wet)) if wet > 0 => {
                info!("[NVS] Overriding soil wet_adc from dedicated NVS key: {} mV", wet);
                config.gauges.soil_moisture.wet_adc = wet;
            }
            _ => {}
        }

        config
    }

    /// Serializes and commits `AppConfig` to NVS flash.
    pub fn save(&self, config: &AppConfig) -> Result<(), String> {
        let json_str = serde_json::to_string(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        self.nvs_cfg
            .set_str(NVS_KEY_CONFIG, &json_str)
            .map_err(|e| format!("Failed to write config to NVS: {}", e))?;

        // Also persist dedicated soil calibration keys for bulletproof survivability
        let dry = config.gauges.soil_moisture.dry_adc;
        let wet = config.gauges.soil_moisture.wet_adc;
        if dry > 0 {
            let _ = self.nvs_cfg.set_i32(NVS_KEY_SOIL_DRY, dry);
        }
        if wet > 0 {
            let _ = self.nvs_cfg.set_i32(NVS_KEY_SOIL_WET, wet);
        }

        info!("[NVS] Configuration saved to flash (dry_adc: {} mV, wet_adc: {} mV)", dry, wet);
        Ok(())
    }

    /// Directly updates and commits soil moisture calibration to flash.
    #[allow(dead_code)]
    pub fn save_soil_calibration(&self, dry: Option<i32>, wet: Option<i32>) -> Result<(), String> {
        if let Some(d) = dry {
            self.nvs_cfg
                .set_i32(NVS_KEY_SOIL_DRY, d)
                .map_err(|e| format!("Failed to save soil_dry: {}", e))?;
        }
        if let Some(w) = wet {
            self.nvs_cfg
                .set_i32(NVS_KEY_SOIL_WET, w)
                .map_err(|e| format!("Failed to save soil_wet: {}", e))?;
        }
        info!("[NVS] Dedicated soil calibration committed: dry={:?}, wet={:?}", dry, wet);
        Ok(())
    }

    /// Loads stored Wi-Fi credentials from NVS.
    pub fn load_wifi(&self) -> Option<WifiCredentials> {
        let mut ssid_buf = [0u8; 64];
        let mut pass_buf = [0u8; 64];

        let ssid = match self.nvs_wifi.get_str(NVS_KEY_SSID, &mut ssid_buf) {
            Ok(Some(s)) if !s.trim().is_empty() => s.to_string(),
            _ => return None,
        };

        let password = match self.nvs_wifi.get_str(NVS_KEY_PASS, &mut pass_buf) {
            Ok(Some(p)) => p.to_string(),
            _ => String::new(),
        };

        info!("[NVS] Loaded saved Wi-Fi credentials for SSID: '{}'", ssid);
        Some(WifiCredentials { ssid, password })
    }

    /// Saves new Wi-Fi credentials to NVS flash.
    pub fn save_wifi(&self, ssid: &str, password: &str) -> Result<(), String> {
        self.nvs_wifi
            .set_str(NVS_KEY_SSID, ssid)
            .map_err(|e| format!("Failed to save Wi-Fi SSID to NVS: {}", e))?;

        self.nvs_wifi
            .set_str(NVS_KEY_PASS, password)
            .map_err(|e| format!("Failed to save Wi-Fi password to NVS: {}", e))?;

        info!("[NVS] Saved Wi-Fi credentials for SSID: '{}'", ssid);
        Ok(())
    }

    /// Clears Wi-Fi credentials from NVS flash (forces setup portal on next boot).
    pub fn clear_wifi(&self) -> Result<(), String> {
        self.nvs_wifi
            .erase_all()
            .map_err(|e| format!("Failed to erase Wi-Fi credentials: {}", e))?;
        info!("[NVS] Cleared Wi-Fi credentials from NVS");
        Ok(())
    }

    /// Loads persisted sensor history from NVS into a RingBuffer (clamped to `MAX_HISTORY_CAPACITY`).
    pub fn load_history(&self, capacity: usize) -> RingBuffer<SensorReading> {
        let effective_capacity = capacity.clamp(1, MAX_HISTORY_CAPACITY);
        let max_bytes = effective_capacity * SensorReading::BINARY_SIZE;
        let mut buf = vec![0u8; max_bytes];

        match self.nvs_log.get_blob(NVS_KEY_HISTORY, &mut buf) {
            Ok(Some(slice)) => {
                let rec_size = SensorReading::BINARY_SIZE;
                let count = slice.len() / rec_size;
                let mut items = Vec::with_capacity(count);
                for chunk in slice.chunks_exact(rec_size) {
                    if let Some(r) = SensorReading::from_binary(chunk) {
                        items.push(r);
                    }
                }
                info!("[NVS] Loaded {} historical sensor readings from flash", items.len());
                RingBuffer::from_items(items, effective_capacity)
            }
            Ok(None) => {
                info!("[NVS] No historical sensor readings found in flash. Starting fresh buffer.");
                RingBuffer::new(effective_capacity)
            }
            Err(e) => {
                warn!("[NVS] Failed to read historical sensor readings: {}. Starting fresh.", e);
                RingBuffer::new(effective_capacity)
            }
        }
    }

    /// Saves the current RingBuffer history to NVS flash, enforcing `MAX_HISTORY_BLOB_BYTES`.
    ///
    /// If NVS reports `ESP_ERR_NVS_NOT_ENOUGH_SPACE` during copy-on-write update,
    /// removes the old blob first to free space and retries, falling back to the
    /// newest half of the buffer if the partition is still too full.
    pub fn save_history(&self, history: &RingBuffer<SensorReading>) -> Result<(), String> {
        let items = history.all();
        let (blob, truncated) = SensorReading::encode_history_bounded(&items);
        if truncated {
            warn!(
                "[NVS] History exceeded {} samples ({}); truncated oldest entries before flash write",
                MAX_HISTORY_CAPACITY,
                items.len()
            );
        }
        if blob.len() > MAX_HISTORY_BLOB_BYTES {
            return Err(format!(
                "Refusing to write oversized history blob ({} B > {} B limit)",
                blob.len(),
                MAX_HISTORY_BLOB_BYTES
            ));
        }

        match self.nvs_log.set_blob(NVS_KEY_HISTORY, &blob) {
            Ok(()) => {
                debug!("[NVS] Committed {} telemetry samples ({} B) to flash", blob.len() / SensorReading::BINARY_SIZE, blob.len());
                Ok(())
            }
            Err(e) if e.code() == esp_idf_svc::sys::ESP_ERR_NVS_NOT_ENOUGH_SPACE => {
                warn!("[NVS] Low NVS space writing {} B history; erasing old blob and retrying", blob.len());
                let _ = self.nvs_log.remove(NVS_KEY_HISTORY);
                match self.nvs_log.set_blob(NVS_KEY_HISTORY, &blob) {
                    Ok(()) => Ok(()),
                    Err(e2) if e2.code() == esp_idf_svc::sys::ESP_ERR_NVS_NOT_ENOUGH_SPACE => {
                        let rec_size = SensorReading::BINARY_SIZE;
                        let half_samples = (blob.len() / rec_size) / 2;
                        let offset = blob.len() - (half_samples * rec_size);
                        let reduced = &blob[offset..];
                        warn!(
                            "[NVS] Still low on NVS space; trimming history to newest {} samples ({} B)",
                            half_samples,
                            reduced.len()
                        );
                        self.nvs_log
                            .set_blob(NVS_KEY_HISTORY, reduced)
                            .map_err(|e3| format!("Failed to write trimmed history blob to NVS: {}", e3))
                    }
                    Err(e2) => Err(format!("Failed to write history blob to NVS: {}", e2)),
                }
            }
            Err(e) => Err(format!("Failed to write history blob to NVS: {}", e)),
        }
    }
}
