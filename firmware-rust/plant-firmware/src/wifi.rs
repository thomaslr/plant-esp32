//! Wi-Fi STA Initialization and Connection Management.
//!
//! Configures modem sleep (WIFI_PS_MIN_MODEM) to minimize dynamic power draw.

use std::ffi::CString;

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys::{
    esp_netif_get_handle_from_ifkey, esp_netif_set_hostname, esp_wifi_set_ps,
    wifi_ps_type_t_WIFI_PS_MIN_MODEM,
};
use esp_idf_svc::wifi::{
    AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi,
};
use log::{info, warn};

pub fn init_wifi(
    modem: Modem<'static>,
    sys_loop: EspSystemEventLoop,
    nvs: Option<EspDefaultNvsPartition>,
    ssid: &str,
    password: &str,
    hostname: &str,
) -> Result<BlockingWifi<EspWifi<'static>>, esp_idf_svc::sys::EspError> {
    info!("[WIFI] Initializing Wi-Fi client for SSID: '{}'", ssid);

    let esp_wifi = EspWifi::new(modem, sys_loop.clone(), nvs)?;
    let mut wifi = BlockingWifi::wrap(esp_wifi, sys_loop)?;

    let wifi_configuration = Configuration::Client(ClientConfiguration {
        ssid: ssid.try_into().unwrap_or_default(),
        password: password.try_into().unwrap_or_default(),
        auth_method: if password.is_empty() {
            AuthMethod::None
        } else {
            AuthMethod::WPA2Personal
        },
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration)?;
    wifi.start()?;
    info!("[WIFI] Driver started");

    // Enable Wi-Fi Modem Sleep (powers down radio between router DTIM beacon intervals)
    unsafe {
        let _ = esp_wifi_set_ps(wifi_ps_type_t_WIFI_PS_MIN_MODEM);
    }
    info!("[PWR] Wi-Fi Modem Sleep enabled (WIFI_PS_MIN_MODEM)");

    // Set hostname on netif
    if let Ok(c_hostname) = CString::new(hostname) {
        unsafe {
            let ifkey = CString::new("WIFI_STA_DEF").unwrap();
            let netif = esp_netif_get_handle_from_ifkey(ifkey.as_ptr());
            if !netif.is_null() {
                let _ = esp_netif_set_hostname(netif, c_hostname.as_ptr());
            }
        }
    }

    match wifi.connect() {
        Ok(_) => {
            info!("[WIFI] Connected to AP successfully");
            match wifi.wait_netif_up() {
                Ok(_) => {
                    let ip = wifi.wifi().sta_netif().get_ip_info();
                    if let Ok(info) = ip {
                        info!("[WIFI] IP Address: {}", info.ip);
                    }
                }
                Err(e) => {
                    warn!("[WIFI] Waiting for DHCP IP timed out: {:?}", e);
                }
            }
        }
        Err(e) => {
            warn!("[WIFI] Initial connection attempt failed ({:?}). Will retry in background.", e);
        }
    }

    Ok(wifi)
}
