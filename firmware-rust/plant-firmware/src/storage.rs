//! Non-Volatile Flash Configuration Storage using ESP32 NVS.
//!
//! Stores `AppConfig` as a JSON string under the `plant_cfg` namespace.

use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};
use log::{error, info, warn};
use plant_core::config::AppConfig;

const NVS_NAMESPACE: &str = "plant_cfg";
const NVS_KEY: &str = "config_json";
const BUFFER_SIZE: usize = 2048;

pub struct NvsConfigStorage {
    nvs: EspNvs<NvsDefault>,
}

impl NvsConfigStorage {
    /// Opens the NVS partition with read/write access.
    pub fn new(partition: EspDefaultNvsPartition) -> Result<Self, esp_idf_svc::sys::EspError> {
        let nvs = EspNvs::new(partition, NVS_NAMESPACE, true)?;
        Ok(Self { nvs })
    }

    /// Loads the stored `AppConfig`. Falls back to default if not found or invalid.
    pub fn load(&self) -> AppConfig {
        let mut buf = [0u8; BUFFER_SIZE];
        match self.nvs.get_str(NVS_KEY, &mut buf) {
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

        self.nvs
            .set_str(NVS_KEY, &json_str)
            .map_err(|e| format!("Failed to write config to NVS: {}", e))?;

        info!("[NVS] Configuration saved to flash");
        Ok(())
    }
}
