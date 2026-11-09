//! Over-The-Air (OTA) Firmware Update Service.
//!
//! Supports:
//! 1. Remote GitHub Releases HTTPS streaming download with automatic redirect following.
//! 2. Direct browser binary upload via REST API.
//! 3. Automatic rollback cancellation (`mark_running_app_valid`).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use embedded_svc::http::Method;
use embedded_svc::io::Read;
use esp_idf_svc::http::client::{Configuration as HttpConfig, EspHttpConnection, FollowRedirectsPolicy};
use esp_idf_svc::ota::EspOta;
use log::{info, warn};

pub const DEFAULT_GITHUB_OTA_URL: &str =
    "https://github.com/thomaslr/plant-esp32/releases/latest/download/plant-firmware.bin";

pub const DEFAULT_GITHUB_VERSION_URL: &str =
    "https://github.com/thomaslr/plant-esp32/releases/latest/download/version.json";

#[derive(Clone, Debug, serde::Serialize)]
pub struct OtaCheckResult {
    pub latest_version: String,
    pub update_available: bool,
    pub last_check_unix: i64,
    pub url: String,
}

static LATEST_OTA_CHECK: Mutex<Option<OtaCheckResult>> = Mutex::new(None);

pub fn get_latest_ota_check() -> Option<OtaCheckResult> {
    LATEST_OTA_CHECK.lock().ok().and_then(|guard| guard.clone())
}

static OTA_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// Returns true when an OTA download or binary flash operation is actively executing.
pub fn is_ota_in_progress() -> bool {
    OTA_IN_PROGRESS.load(Ordering::Relaxed)
}

struct OtaGuard;
impl Drop for OtaGuard {
    fn drop(&mut self) {
        OTA_IN_PROGRESS.store(false, Ordering::SeqCst);
    }
}

/// Details about running and available OTA partitions.
#[derive(serde::Serialize, Clone, Debug)]
pub struct OtaStatus {
    pub running_slot: String,
    pub next_update_slot: String,
    pub firmware_version: &'static str,
    pub default_url: &'static str,
    pub rollback_enabled: bool,
    pub update_available: bool,
    pub latest_firmware_version: Option<String>,
    pub last_check_unix: Option<i64>,
}

/// Marks the currently running slot as valid.
///
/// Under `CONFIG_BOOTLOADER_APP_ROLLBACK_ENABLE=y`, this confirms that the
/// newly booted image reached full operational state without panicking,
/// permanently cancelling automatic bootloader rollback.
pub fn mark_running_app_valid() {
    match EspOta::new() {
        Ok(mut ota) => match ota.mark_running_slot_valid() {
            Ok(_) => {
                info!("[OTA] Boot verification confirmed: running slot marked valid.");
            }
            Err(e) => {
                info!("[OTA] Running slot status (already valid or factory): {:?}", e);
            }
        },
        Err(e) => {
            warn!("[OTA] Unable to access OTA service: {:?}", e);
        }
    }
}

/// Returns details about current running slot and target update slot.
pub fn get_ota_status() -> OtaStatus {
    let mut running_slot = "factory".to_string();
    let mut next_update_slot = "none".to_string();

    if let Ok(ota) = EspOta::new() {
        if let Ok(slot) = ota.get_running_slot() {
            running_slot = slot.label.as_str().to_string();
        }
        if let Ok(slot) = ota.get_update_slot() {
            next_update_slot = slot.label.as_str().to_string();
        }
    }

    let (update_available, latest_firmware_version, last_check_unix) = match get_latest_ota_check() {
        Some(chk) => (chk.update_available, Some(chk.latest_version), Some(chk.last_check_unix)),
        None => (false, None, None),
    };

    OtaStatus {
        running_slot,
        next_update_slot,
        firmware_version: env!("CARGO_PKG_VERSION"),
        default_url: DEFAULT_GITHUB_OTA_URL,
        rollback_enabled: true,
        update_available,
        latest_firmware_version,
        last_check_unix,
    }
}

/// Queries GitHub Releases for the lightweight version.json metadata file.
pub fn check_latest_version() -> Result<OtaCheckResult, String> {
    if is_ota_in_progress() {
        return Err("OTA update currently in progress".to_string());
    }

    info!("[OTA] Autonomous check: querying {}", DEFAULT_GITHUB_VERSION_URL);

    let config = HttpConfig {
        follow_redirects_policy: FollowRedirectsPolicy::FollowGetHead,
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        buffer_size: Some(4096),
        buffer_size_tx: Some(2048),
        timeout: Some(Duration::from_secs(15)),
        ..Default::default()
    };

    let mut conn = EspHttpConnection::new(&config)
        .map_err(|e| format!("Failed to initialize HTTP connection: {:?}", e))?;

    conn.initiate_request(Method::Get, DEFAULT_GITHUB_VERSION_URL, &[("User-Agent", "plant-esp32-ota")])
        .map_err(|e| format!("Failed to initiate request: {:?}", e))?;

    conn.initiate_response()
        .map_err(|e| format!("Failed to initiate response: {:?}", e))?;

    let status = conn.status();
    if !(200..300).contains(&status) {
        return Err(format!("Server returned HTTP {}", status));
    }

    let mut buf = [0u8; 1024];
    let mut total_read = 0;
    loop {
        match conn.read(&mut buf[total_read..]) {
            Ok(0) => break,
            Ok(n) => {
                total_read += n;
                if total_read >= buf.len() {
                    break;
                }
            }
            Err(e) => return Err(format!("Read error: {:?}", e)),
        }
    }

    let v: serde_json::Value = serde_json::from_slice(&buf[..total_read])
        .map_err(|e| format!("Failed to parse version.json: {:?}", e))?;

    let latest_ver = v.get("version")
        .and_then(|val| val.as_str())
        .ok_or_else(|| "Missing 'version' key in version.json".to_string())?
        .to_string();

    let download_url = v.get("url")
        .and_then(|val| val.as_str())
        .unwrap_or(DEFAULT_GITHUB_OTA_URL)
        .to_string();

    let current_ver = env!("CARGO_PKG_VERSION");
    let update_available = plant_core::is_version_newer(&latest_ver, current_ver);

    let now_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let result = OtaCheckResult {
        latest_version: latest_ver.clone(),
        update_available,
        last_check_unix: now_unix,
        url: download_url,
    };

    info!(
        "[OTA] Autonomous check complete: installed='{}', latest='{}', update_available={}",
        current_ver, latest_ver, update_available
    );

    if let Ok(mut lock) = LATEST_OTA_CHECK.lock() {
        *lock = Some(result.clone());
    }

    Ok(result)
}

/// Spawns the autonomous background worker for OTA update checks.
/// - Checks shortly after boot (~60 seconds after startup).
/// - Checks once per day sometime after midnight (~02:00 AM local time).
pub fn start_periodic_ota_check<F>(gmt_offset_fn: F) -> std::io::Result<()>
where
    F: Fn() -> i32 + Send + 'static,
{
    std::thread::Builder::new()
        .name("OtaCheckWorker".to_string())
        .stack_size(24 * 1024)
        .spawn(move || {
            // Initial delay after boot to let Wi-Fi and SNTP settle
            info!("[OTA] Autonomous check worker initialized; will run initial check in 60s");
            std::thread::sleep(Duration::from_secs(60));

            // 1. Shortly after boot check:
            if let Err(e) = check_latest_version() {
                warn!("[OTA] Boot update check failed: {}", e);
            }

            let mut last_checked_day: Option<i64> = None;

            loop {
                std::thread::sleep(Duration::from_secs(60));

                let now_unix = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);

                let gmt_offset = gmt_offset_fn();

                if let (Some(hour), Some(day)) = (
                    plant_core::timeutil::local_hour(now_unix, gmt_offset),
                    plant_core::timeutil::local_day(now_unix, gmt_offset),
                ) {
                    // 2. Once per day sometime after midnight (at 02:00 local time):
                    if hour == 2 && last_checked_day != Some(day) {
                        last_checked_day = Some(day);
                        info!("[OTA] Nightly post-midnight update check running (local hour: 2 AM)...");
                        if let Err(e) = check_latest_version() {
                            warn!("[OTA] Nightly update check failed: {}", e);
                        }
                    }
                }
            }
        })?;

    Ok(())
}

/// Streams a new firmware image from an HTTPS URL (e.g. GitHub Releases) into the next OTA slot.
pub fn update_from_url(url: &str) -> Result<(), String> {
    info!("[OTA] Starting OTA download from: {}", url);
    OTA_IN_PROGRESS.store(true, Ordering::SeqCst);
    let _guard = OtaGuard;

    let mut ota = EspOta::new().map_err(|e| format!("Failed to initialize OTA: {:?}", e))?;
    let mut update = ota
        .initiate_update()
        .map_err(|e| format!("Failed to initiate OTA update: {:?}", e))?;

    // GitHub Releases download requires following 2 redirects, ending in a ~1.2 KB
    // signed Azure/AWS S3 blob URL with long auth tokens.
    // buffer_size_tx must be large enough (e.g. 4096 bytes) so esp_http_client
    // does not error with "HTTP_CLIENT: Out of buffer".
    let config = HttpConfig {
        follow_redirects_policy: FollowRedirectsPolicy::FollowGetHead,
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        buffer_size: Some(8192),
        buffer_size_tx: Some(4096),
        timeout: Some(Duration::from_secs(60)),
        ..Default::default()
    };

    let mut conn = EspHttpConnection::new(&config)
        .map_err(|e| format!("Failed to create HTTP client: {:?}", e))?;

    conn.initiate_request(Method::Get, url, &[("User-Agent", "plant-esp32-ota")])
        .map_err(|e| format!("Failed to initiate HTTP GET: {:?}", e))?;

    conn.initiate_response()
        .map_err(|e| format!("Failed to get HTTP response: {:?}", e))?;

    let status = conn.status();
    if !(200..300).contains(&status) {
        let _ = update.abort();
        return Err(format!("Server returned HTTP {}", status));
    }

    // Heap-allocate the chunk buffer rather than burning stack space.
    let mut buf = vec![0u8; 4096];
    let mut total_bytes = 0usize;
    let mut last_log_bytes = 0usize;

    loop {
        match conn.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                update
                    .write(&buf[..n])
                    .map_err(|e| format!("Failed writing flash chunk: {:?}", e))?;
                total_bytes += n;
                if total_bytes - last_log_bytes >= 102400 {
                    info!("[OTA] Downloaded and flashed {} KB...", total_bytes / 1024);
                    last_log_bytes = total_bytes;
                }
            }
            Err(e) => {
                let _ = update.abort();
                return Err(format!("Download error after {} bytes: {:?}", total_bytes, e));
            }
        }
    }

    if total_bytes < 500_000 {
        let _ = update.abort();
        return Err(format!(
            "Downloaded file size ({} bytes) too small to be valid firmware",
            total_bytes
        ));
    }

    info!(
        "[OTA] Download complete ({} bytes). Finalizing flash partition...",
        total_bytes
    );

    update
        .complete()
        .map_err(|e| format!("Failed to complete OTA: {:?}", e))?;

    info!("[OTA] Firmware written successfully. Restarting in 2 seconds...");
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(2000));
        unsafe {
            esp_idf_svc::sys::esp_restart();
        }
    });

    Ok(())
}

/// Writes incoming binary data chunks (e.g. from local browser file upload) to the OTA partition.
pub fn flash_stream<R: Read>(mut reader: R, expected_len: usize) -> Result<(), String> {
    info!("[OTA] Flashing firmware from stream (expected: {} bytes)...", expected_len);
    OTA_IN_PROGRESS.store(true, Ordering::SeqCst);
    let _guard = OtaGuard;

    let mut ota = EspOta::new().map_err(|e| format!("Failed to initialize OTA: {:?}", e))?;
    let mut update = ota
        .initiate_update()
        .map_err(|e| format!("Failed to initiate OTA update: {:?}", e))?;

    let mut buf = [0u8; 2048];
    let mut total_bytes = 0usize;

    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                update
                    .write(&buf[..n])
                    .map_err(|e| format!("Failed writing flash: {:?}", e))?;
                total_bytes += n;
            }
            Err(e) => {
                let _ = update.abort();
                return Err(format!("Stream read error: {:?}", e));
            }
        }
    }

    if total_bytes < 500_000 {
        let _ = update.abort();
        return Err(format!(
            "Uploaded file ({} bytes) too small to be valid firmware",
            total_bytes
        ));
    }

    update
        .complete()
        .map_err(|e| format!("Failed to complete OTA: {:?}", e))?;

    info!("[OTA] Upload complete ({} bytes). Restarting in 2 seconds...", total_bytes);
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(2000));
        unsafe {
            esp_idf_svc::sys::esp_restart();
        }
    });

    Ok(())
}
