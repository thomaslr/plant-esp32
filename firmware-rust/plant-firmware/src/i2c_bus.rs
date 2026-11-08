//! Hardware I2C Driver for SHT30 & BH1750 on ESP32-C3 SuperMini.
//!
//! SCL: GPIO 5
//! SDA: GPIO 6

use esp_idf_svc::sys::{
    i2c_config_t, i2c_config_t__bindgen_ty_1, i2c_config_t__bindgen_ty_1__bindgen_ty_1,
    i2c_driver_install, i2c_master_read_from_device, i2c_master_write_to_device,
    i2c_mode_t_I2C_MODE_MASTER, i2c_param_config, i2c_port_t_I2C_NUM_0, ESP_OK,
};
use log::{debug, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

use crate::pins::{PIN_I2C_SCL, PIN_I2C_SDA};

static I2C_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Standard 7-bit I2C addresses for supported peripherals.
pub const SHT30_ADDR_PRIMARY: u8 = 0x44;
pub const SHT30_ADDR_SECONDARY: u8 = 0x45;

pub const BH1750_ADDR_PRIMARY: u8 = 0x23;
pub const BH1750_ADDR_SECONDARY: u8 = 0x5C;

/// Initializes the I2C Master peripheral if not already initialized.
pub fn ensure_i2c_initialized() -> bool {
    if I2C_INITIALIZED.load(Ordering::SeqCst) {
        return true;
    }

    let mut conf = i2c_config_t::default();
    conf.mode = i2c_mode_t_I2C_MODE_MASTER;
    conf.sda_io_num = PIN_I2C_SDA;
    conf.scl_io_num = PIN_I2C_SCL;
    conf.sda_pullup_en = true;
    conf.scl_pullup_en = true;
    conf.__bindgen_anon_1 = i2c_config_t__bindgen_ty_1 {
        master: i2c_config_t__bindgen_ty_1__bindgen_ty_1 { clk_speed: 100_000 },
    };

    unsafe {
        let err = i2c_param_config(i2c_port_t_I2C_NUM_0, &conf);
        if err != ESP_OK {
            warn!("[I2C] Failed to configure I2C parameters: {}", err);
            return false;
        }

        let err = i2c_driver_install(i2c_port_t_I2C_NUM_0, conf.mode, 0, 0, 0);
        if err != ESP_OK {
            warn!("[I2C] Failed to install I2C driver: {}", err);
            return false;
        }
    }

    I2C_INITIALIZED.store(true, Ordering::SeqCst);
    debug!("[I2C] Driver initialized on SDA=GPIO{}, SCL=GPIO{} (100kHz)", PIN_I2C_SDA, PIN_I2C_SCL);
    true
}

/// SHT30 CRC-8 calculation (polynomial 0x31, init 0xFF).
fn sht30_crc8(data: &[u8]) -> u8 {
    let mut crc: u8 = 0xFF;
    for &byte in data {
        crc ^= byte;
        for _ in 0..8 {
            if (crc & 0x80) != 0 {
                crc = (crc << 1) ^ 0x31;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// Reads temperature (°C) and relative humidity (%) from SHT30.
/// Tries address 0x44, then 0x45 fallback.
pub fn read_sht30() -> Option<(f32, f32)> {
    if !ensure_i2c_initialized() {
        return None;
    }

    for &addr in &[SHT30_ADDR_PRIMARY, SHT30_ADDR_SECONDARY] {
        // High repeatability measurement command with clock stretching disabled: 0x2400
        let cmd = [0x24, 0x00];
        let mut buf = [0u8; 6];

        unsafe {
            let write_err = i2c_master_write_to_device(
                i2c_port_t_I2C_NUM_0,
                addr,
                cmd.as_ptr(),
                cmd.len(),
                100, // 100 FreeRTOS ticks timeout
            );
            if write_err != ESP_OK {
                continue;
            }
        }

        // Wait 15ms for measurement conversion
        sleep(Duration::from_millis(15));

        unsafe {
            let read_err = i2c_master_read_from_device(
                i2c_port_t_I2C_NUM_0,
                addr,
                buf.as_mut_ptr(),
                buf.len(),
                100,
            );
            if read_err != ESP_OK {
                continue;
            }
        }

        // Verify CRC8 on temperature (bytes 0-1 vs byte 2)
        if sht30_crc8(&buf[0..2]) != buf[2] {
            warn!("[SHT30] Temperature CRC mismatch on 0x{:02X}", addr);
            continue;
        }

        // Verify CRC8 on humidity (bytes 3-4 vs byte 5)
        if sht30_crc8(&buf[3..5]) != buf[5] {
            warn!("[SHT30] Humidity CRC mismatch on 0x{:02X}", addr);
            continue;
        }

        let raw_temp = u16::from_be_bytes([buf[0], buf[1]]) as f32;
        let raw_humi = u16::from_be_bytes([buf[3], buf[4]]) as f32;

        let temp_c = -45.0 + 175.0 * (raw_temp / 65535.0);
        let humi_pct = 100.0 * (raw_humi / 65535.0);

        debug!("[SHT30] 0x{:02X} -> Temp: {:.1}°C, Humidity: {:.1}%", addr, temp_c, humi_pct);
        return Some((temp_c, humi_pct));
    }

    None
}

/// Reads ambient light in Lux from BH1750.
/// Tries address 0x23, then 0x5C fallback.
pub fn read_bh1750() -> Option<f32> {
    if !ensure_i2c_initialized() {
        return None;
    }

    for &addr in &[BH1750_ADDR_PRIMARY, BH1750_ADDR_SECONDARY] {
        // One-time High Resolution mode (0x20): 1 lux resolution, 120ms measurement time, auto power-down
        let cmd = [0x20u8];
        let mut buf = [0u8; 2];

        unsafe {
            let write_err = i2c_master_write_to_device(
                i2c_port_t_I2C_NUM_0,
                addr,
                cmd.as_ptr(),
                cmd.len(),
                100,
            );
            if write_err != ESP_OK {
                continue;
            }
        }

        // Wait 130ms for measurement completion
        sleep(Duration::from_millis(130));

        unsafe {
            let read_err = i2c_master_read_from_device(
                i2c_port_t_I2C_NUM_0,
                addr,
                buf.as_mut_ptr(),
                buf.len(),
                100,
            );
            if read_err != ESP_OK {
                continue;
            }
        }

        let raw = u16::from_be_bytes([buf[0], buf[1]]) as f32;
        // BH1750 formula: lux = raw / 1.2
        let lux = raw / 1.2;

        debug!("[BH1750] 0x{:02X} -> Light: {:.1} lux", addr, lux);
        return Some(lux);
    }

    None
}
