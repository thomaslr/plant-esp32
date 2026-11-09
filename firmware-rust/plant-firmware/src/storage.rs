//! Non-Volatile Flash Configuration & Wi-Fi Storage using ESP32 NVS.
//!
//! Stores `AppConfig` under the `plant_cfg` namespace and
//! network credentials under the `plant_wifi` namespace.

use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};
use log::{debug, error, info, warn};
use plant_core::config::AppConfig;
use plant_core::reading::SensorReading;
use plant_core::ring_buffer::RingBuffer;

const CFG_NAMESPACE: &str = "plant_cfg";
const NVS_KEY_CONFIG: &str = "config_json";

const WIFI_NAMESPACE: &str = "plant_wifi";
const NVS_KEY_SSID: &str = "wifi_ssid";
const NVS_KEY_PASS: &str = "wifi_pass";

const LOG_NAMESPACE: &str = "plant_log";
const NVS_KEY_HISTORY: &str = "hist_bin";

const BUFFER_SIZE: usize = 2048;

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
        match self.nvs_cfg.get_str(NVS_KEY_CONFIG, &mut buf) {
            Ok(Some(json_str)) => match serde_json::from_str::<AppConfig>(json_str) {
                Ok(cfg) => {
                    info!("[NVS] Loaded existing configuration from flash");
                    cfg
                }
                Err(e) => {
                    warn!("[NVS] Corrupt config JSON in NVS ({}). Falling back to defaults.", e);
                    let def = AppConfig::default();
                    let _ = self.save(&def);
                    def
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
        }
    }

    /// Serializes and commits `AppConfig` to NVS flash.
    pub fn save(&self, config: &AppConfig) -> Result<(), String> {
        let json_str = serde_json::to_string(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        self.nvs_cfg
            .set_str(NVS_KEY_CONFIG, &json_str)
            .map_err(|e| format!("Failed to write config to NVS: {}", e))?;

        info!("[NVS] Configuration saved to flash");
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

    /// Loads persisted sensor history from NVS into a RingBuffer (up to capacity 100).
    pub fn load_history(&self, capacity: usize) -> RingBuffer<SensorReading> {
        let max_bytes = capacity * SensorReading::BINARY_SIZE;
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
                RingBuffer::from_items(items, capacity)
            }
            Ok(None) => {
                info!("[NVS] No historical sensor readings found in flash. Starting fresh buffer.");
                RingBuffer::new(capacity)
            }
            Err(e) => {
                warn!("[NVS] Failed to read historical sensor readings: {}. Starting fresh.", e);
                RingBuffer::new(capacity)
            }
        }
    }

    /// Saves the current RingBuffer history to NVS flash.
    pub fn save_history(&self, history: &RingBuffer<SensorReading>) -> Result<(), String> {
        let items = history.all();
        let mut blob = Vec::with_capacity(items.len() * SensorReading::BINARY_SIZE);
        for item in &items {
            blob.extend_from_slice(&item.to_binary());
        }

        self.nvs_log
            .set_blob(NVS_KEY_HISTORY, &blob)
            .map_err(|e| format!("Failed to write history blob to NVS: {}", e))?;

        debug!("[NVS] Committed {} telemetry samples to flash", items.len());
        Ok(())
    }
}
