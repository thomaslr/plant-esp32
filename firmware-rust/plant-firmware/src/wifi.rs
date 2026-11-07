//! Wi-Fi Station and SoftAP Provisioning Management.
//!
//! Supports two modes:
//! 1. Station Mode: Connects to saved home/office router with modem sleep enabled.
//! 2. Provisioning SoftAP Mode: Broadcasts `PlantMonitor-Setup-XXXX` (IP 192.168.4.1)
//!    with a captive portal allowing users/friends to configure Wi-Fi credentials from any phone.

use std::ffi::CString;

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys::{
    esp_netif_get_handle_from_ifkey, esp_netif_set_hostname, esp_wifi_get_max_tx_power,
    esp_wifi_set_max_tx_power, esp_wifi_set_ps, wifi_ps_type_t_WIFI_PS_NONE,
};
use esp_idf_svc::wifi::{
    AccessPointConfiguration, AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi,
};
use log::{info, warn};

use crate::storage::WifiCredentials;

/// Operational Wi-Fi state determined during boot.
pub enum WifiState {
    /// Connected to router in Station mode.
    StationConnected {
        ip_address: String,
        wifi_handle: BlockingWifi<EspWifi<'static>>,
    },
    /// Running SoftAP for initial captive portal provisioning.
    ProvisioningMode {
        ap_ssid: String,
        scanned_ssids: Vec<String>,
        wifi_handle: BlockingWifi<EspWifi<'static>>,
    },
}

/// Initializes Wi-Fi hardware and connects to saved network, or launches SoftAP provisioning.
pub fn start_wifi(
    modem: Modem<'static>,
    sys_loop: EspSystemEventLoop,
    nvs: Option<EspDefaultNvsPartition>,
    saved_creds: Option<WifiCredentials>,
    hostname: &str,
) -> Result<WifiState, esp_idf_svc::sys::EspError> {
    let esp_wifi = EspWifi::new(modem, sys_loop.clone(), nvs)?;
    let mut wifi = BlockingWifi::wrap(esp_wifi, sys_loop)?;

    // 1. If saved credentials exist, attempt Station Mode first
    if let Some(creds) = saved_creds {
        info!("[WIFI] Attempting connection to saved network: '{}'", creds.ssid);

        let client_config = ClientConfiguration {
            ssid: creds.ssid.as_str().try_into().unwrap_or_default(),
            password: creds.password.as_str().try_into().unwrap_or_default(),
            auth_method: if creds.password.is_empty() {
                AuthMethod::None
            } else {
                AuthMethod::WPA2Personal
            },
            ..Default::default()
        };

        if let Ok(()) = wifi.set_configuration(&Configuration::Client(client_config)) {
            if let Ok(()) = wifi.start() {
                // Boost Wi-Fi transmission power to maximum (78 = ~19.5-20 dBm, ~100mW)
                // to optimize signal reach with wall-powered operation.
                unsafe {
                    let _ = esp_wifi_set_max_tx_power(78);
                    let mut actual_pwr: i8 = 0;
                    if esp_wifi_get_max_tx_power(&mut actual_pwr) == 0 {
                        info!("[WIFI] Maximum TX power configured: {} (quarter-dBm, ~{:.1} dBm)", actual_pwr, (actual_pwr as f32) / 4.0);
                    }
                    // Keep modem power save OFF during initial connection/DHCP handshake
                    // to prevent packet drops and auth timeouts with mesh routers (e.g. Deco).
                    let _ = esp_wifi_set_ps(wifi_ps_type_t_WIFI_PS_NONE);
                }
                set_sta_hostname(hostname);

                // A single failed attempt is common right after a crash/OTA reboot: the
                // router can still hold a stale session for this MAC and rejects the first
                // auth (reason 6/0x600). Mesh systems like TP-Link Deco take time to clear
                // stale leases and route DHCP packets. Retry up to 10 times with 15s delays.
                const MAX_STA_ATTEMPTS: u32 = 10;
                for attempt in 1..=MAX_STA_ATTEMPTS {
                    if attempt > 1 {
                        warn!(
                            "[WIFI] Retrying saved network (attempt {}/{} in 15s with radio cycle)...",
                            attempt, MAX_STA_ATTEMPTS
                        );
                        let _ = wifi.disconnect();
                        let _ = wifi.stop();
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        let _ = wifi.start();
                        unsafe {
                            let _ = esp_wifi_set_max_tx_power(78);
                            let _ = esp_wifi_set_ps(wifi_ps_type_t_WIFI_PS_NONE);
                        }
                        std::thread::sleep(std::time::Duration::from_secs(14));
                    }

                    if let Ok(()) = wifi.connect() {
                        info!("[WIFI] Associated with AP. Settling 750ms before DHCP negotiation (ESPEasy connect wait)...");
                        std::thread::sleep(std::time::Duration::from_millis(750));
                        if let Ok(()) = wifi.wait_netif_up() {
                            if let Ok(info) = wifi.wifi().sta_netif().get_ip_info() {
                                let ip_str = info.ip.to_string();
                                if ip_str != "0.0.0.0" {
                                    info!("[WIFI] Station Mode active! IP: {}", ip_str);
                                    // Keep modem sleep permanently OFF (WIFI_PS_NONE) since device is wall-powered.
                                    // Prevents mesh APs (e.g. TP-Link Deco) from dropping unicast frames to sleeping stations.
                                    unsafe {
                                        let _ = esp_wifi_set_ps(wifi_ps_type_t_WIFI_PS_NONE);
                                    }
                                    info!("[WIFI] Modem sleep disabled (WIFI_PS_NONE 24/7) for zero-latency mesh reachability");
                                    return Ok(WifiState::StationConnected {
                                        ip_address: ip_str,
                                        wifi_handle: wifi,
                                    });
                                }
                            }
                        }
                    }
                }
                warn!("[WIFI] Failed to connect to saved network after {} attempts (router offline or credentials changed)", MAX_STA_ATTEMPTS);
                let _ = wifi.stop();
            }
        }
    }

    // 2. Fall back to Provisioning SoftAP Mode
    let mut mac = [0u8; 6];
    unsafe {
        esp_idf_svc::sys::esp_read_mac(
            mac.as_mut_ptr(),
            esp_idf_svc::sys::esp_mac_type_t_ESP_MAC_WIFI_STA,
        );
    }
    let ap_ssid = format!("PlantMonitor-Setup-{:02X}{:02X}", mac[4], mac[5]);
    info!("[WIFI] Starting Provisioning SoftAP: '{}'", ap_ssid);

    let ap_config = AccessPointConfiguration {
        ssid: ap_ssid.as_str().try_into().unwrap_or_default(),
        channel: 1,
        auth_method: AuthMethod::None,
        max_connections: 4,
        ..Default::default()
    };

    // Use Mixed mode (Client + AP) so client radio can scan nearby SSIDs while SoftAP is hosted
    let mixed_config = Configuration::Mixed(ClientConfiguration::default(), ap_config);
    if let Err(e) = wifi.set_configuration(&mixed_config) {
        warn!("[WIFI] Mixed mode configuration failed ({:?}), falling back to AccessPoint only", e);
        let ap_only = Configuration::AccessPoint(AccessPointConfiguration {
            ssid: ap_ssid.as_str().try_into().unwrap_or_default(),
            channel: 1,
            auth_method: AuthMethod::None,
            max_connections: 4,
            ..Default::default()
        });
        wifi.set_configuration(&ap_only)?;
    }

    wifi.start()?;
    info!("[WIFI] SoftAP active. Gateway IP: 192.168.4.1");

    // Scan for nearby 2.4 GHz networks
    info!("[WIFI] Scanning nearby 2.4 GHz networks...");
    let mut scanned_ssids = Vec::new();
    if let Ok(aps) = wifi.scan() {
        for ap in aps {
            let name = ap.ssid.as_str().trim();
            if !name.is_empty() && !scanned_ssids.iter().any(|s: &String| s == name) {
                scanned_ssids.push(name.to_string());
            }
        }
        info!("[WIFI] Discovered {} unique nearby networks", scanned_ssids.len());
    } else {
        warn!("[WIFI] Initial scan returned no results");
    }

    Ok(WifiState::ProvisioningMode {
        ap_ssid,
        scanned_ssids,
        wifi_handle: wifi,
    })
}

fn set_sta_hostname(hostname: &str) {
    if let Ok(c_hostname) = CString::new(hostname) {
        unsafe {
            let ifkey = CString::new("WIFI_STA_DEF").unwrap();
            let netif = esp_netif_get_handle_from_ifkey(ifkey.as_ptr());
            if !netif.is_null() {
                let _ = esp_netif_set_hostname(netif, c_hostname.as_ptr());
            }
        }
    }
}
