// Verify SHT30 CRC8 implementation matches Sensirion specs
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

#[test]
fn test_sht30_crc() {
    assert_eq!(sht30_crc8(&[0xBE, 0xEF]), 0x92);
}

#[test]
fn test_sht30_temp_humidity_conversion() {
    // 0x6666 raw -> temp = -45 + 175 * (26214 / 65535) = 24.99 C
    let raw_temp = 0x6666u16 as f32;
    let temp_c = -45.0 + 175.0 * (raw_temp / 65535.0);
    assert!((temp_c - 25.0).abs() < 0.1);

    // 0x8000 raw -> 50% humidity
    let raw_humi = 0x8000u16 as f32;
    let humi = 100.0 * (raw_humi / 65535.0);
    assert!((humi - 50.0).abs() < 0.1);
}

#[test]
fn test_bh1750_conversion() {
    // 120 raw -> 100 lux
    let raw = 120.0f32;
    let lux = raw / 1.2;
    assert!((lux - 100.0).abs() < 0.001);
}
