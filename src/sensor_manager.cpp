#include "sensor_manager.h"
#include <algorithm>
#include <esp_timer.h>

SensorManager::SensorManager(const AppConfig& config)
    : config_(config) {}

void SensorManager::begin() {
    // Ultrasonic pins
    pinMode(PIN_ULTRASONIC_TRIG, OUTPUT);
    digitalWrite(PIN_ULTRASONIC_TRIG, LOW);
    pinMode(PIN_ULTRASONIC_ECHO, INPUT);

    // Switched power pin for soil moisture sensor: start OFF
    pinMode(PIN_SOIL_POWER, OUTPUT);
    digitalWrite(PIN_SOIL_POWER, LOW);

    // Analog pins configuration (12-bit, 0-3.3V full-scale attenuation)
    analogReadResolution(12);
    analogSetAttenuation(ADC_11db);

    // Initial settle delay
    delay(50);
}

void SensorManager::updateConfig(const AppConfig& config) {
    config_ = config;
}

void SensorManager::forceUltrasonicReset() {
    // Actively clear stuck latch on cheap clone sensors:
    // 1. Force the ECHO pin LOW as an output for 10 microseconds to discharge the line
    pinMode(PIN_ULTRASONIC_ECHO, OUTPUT);
    digitalWrite(PIN_ULTRASONIC_ECHO, LOW);
    delayMicroseconds(10);
    
    // 2. Restore ECHO pin to high-impedance INPUT
    pinMode(PIN_ULTRASONIC_ECHO, INPUT);
    
    // 3. Acoustic quiet time to allow echo reverberations to settle
    delay(30);
    
    preventive_resets_count_++;
}

float SensorManager::readUltrasonicDistanceCm() {
    constexpr size_t NUM_SAMPLES = 3;
    float samples[NUM_SAMPLES];
    size_t valid_samples = 0;

    for (size_t i = 0; i < NUM_SAMPLES; ++i) {
        // Preventive check: If ECHO is stuck HIGH before trigger, clear it immediately
        if (digitalRead(PIN_ULTRASONIC_ECHO) == HIGH) {
            forceUltrasonicReset();
        }

        // Send 10µs TRIG pulse
        digitalWrite(PIN_ULTRASONIC_TRIG, LOW);
        delayMicroseconds(2);
        digitalWrite(PIN_ULTRASONIC_TRIG, HIGH);
        delayMicroseconds(10);
        digitalWrite(PIN_ULTRASONIC_TRIG, LOW);

        // Wait for ECHO pin to go HIGH (max 5ms wait)
        int64_t t_trig = esp_timer_get_time();
        bool started = false;
        while ((esp_timer_get_time() - t_trig) < 5000) {
            if (digitalRead(PIN_ULTRASONIC_ECHO) == HIGH) {
                started = true;
                break;
            }
        }

        if (!started) {
            // Sensor didn't respond to trigger pulse
            delay(10);
            continue;
        }

        // Measure ECHO HIGH duration with a HARD 30ms (30,000µs) timeout
        int64_t t_start = esp_timer_get_time();
        bool ended = false;
        while ((esp_timer_get_time() - t_start) < 30000) {
            if (digitalRead(PIN_ULTRASONIC_ECHO) == LOW) {
                ended = true;
                break;
            }
        }
        int64_t t_end = esp_timer_get_time();

        if (!ended) {
            // ECHO stayed HIGH longer than 30ms -> Echo lost or stuck sensor!
            forceUltrasonicReset();
            continue;
        }

        int64_t duration_us = t_end - t_start;
        float distance = duration_us / 58.2f;

        // Filter unrealistic distances (HC-SR04 valid range is 2cm to 400cm)
        if (distance >= 2.0f && distance <= 400.0f) {
            samples[valid_samples++] = distance;
        }

        // Inter-sample delay for acoustic damping
        delay(20);
    }

    if (valid_samples == 0) {
        return -1.0f; // Timeout or sensor fault
    }

    return calculateMedian(samples, valid_samples);
}

int SensorManager::readSoilMoistureRaw() {
    // 1. Power ON soil sensor probe via GPIO 2
    digitalWrite(PIN_SOIL_POWER, HIGH);

    // 2. Allow 30ms for onboard 555 oscillator & bypass filter capacitor to stabilize
    delay(30);

    constexpr size_t NUM_SAMPLES = 5;
    int samples[NUM_SAMPLES];

    for (size_t i = 0; i < NUM_SAMPLES; ++i) {
        // Read factory eFuse-calibrated millivolts
        samples[i] = analogReadMilliVolts(PIN_SOIL_MOISTURE_ADC);
        delay(5);
    }

    // 3. Power OFF probe immediately to eliminate idle draw and galvanic corrosion
    digitalWrite(PIN_SOIL_POWER, LOW);

    return calculateMedianInt(samples, NUM_SAMPLES);
}

float SensorManager::readSoilMoisturePercent() {
    int raw_mv = readSoilMoistureRaw();
    int dry = config_.gauges.soil.dry_adc;
    int wet = config_.gauges.soil.wet_adc;

    // Capacitive sensors have an INVERTED curve:
    // High voltage/ADC in dry air, low voltage/ADC when submerged in wet soil
    if (dry == wet) return 0.0f;

    float percent = 100.0f * (1.0f - static_cast<float>(raw_mv - wet) / (dry - wet));
    return constrain(percent, 0.0f, 100.0f);
}

float SensorManager::readSupplyVoltage() {
    constexpr size_t NUM_SAMPLES = 5;
    int samples[NUM_SAMPLES];

    for (size_t i = 0; i < NUM_SAMPLES; ++i) {
        samples[i] = analogReadMilliVolts(PIN_SUPPLY_MONITOR_ADC);
        delay(5);
    }

    int median_mv = calculateMedianInt(samples, NUM_SAMPLES);

    // 10k/10k voltage divider steps 5V down by 1/2.
    // Full rail voltage = 2.0 * measured voltage at divider center tap.
    float rail_volts = (median_mv * 2.0f) / 1000.0f;
    return rail_volts;
}

float SensorManager::readChipTemperature() {
    return temperatureRead();
}

SensorReading SensorManager::readAllSensors(bool pumpActiveState) {
    SensorReading r;
    r.timestamp = static_cast<uint32_t>(esp_timer_get_time() / 1000000ULL);
    r.pump_active = pumpActiveState;

    // 1. Supply Rail Voltage
    r.supply_voltage = readSupplyVoltage();

    // 2. Chip Temperature
    r.chip_temp = readChipTemperature();

    // 3. Soil Moisture
    r.soil_raw_adc = readSoilMoistureRaw();
    r.soil_percent = readSoilMoisturePercent();

    // 4. Ultrasonic Water Reservoir Level
    r.distance_cm = readUltrasonicDistanceCm();
    if (r.distance_cm > 0.0f) {
        float tank_h = config_.gauges.water.tank_height_cm;
        float offset = config_.gauges.water.sensor_offset_cm;
        
        // Water level depth = total height - distance to surface
        float depth = tank_h - r.distance_cm;
        r.water_level_cm = constrain(depth, 0.0f, tank_h);

        // Water level percent (usable range between offset and total depth)
        float usable_span = tank_h - offset;
        if (usable_span > 0.0f) {
            float pct = (r.water_level_cm / usable_span) * 100.0f;
            r.water_percent = constrain(pct, 0.0f, 100.0f);
        } else {
            r.water_percent = 0.0f;
        }
    } else {
        r.water_level_cm = 0.0f;
        r.water_percent = 0.0f;
    }

    return r;
}

float SensorManager::calculateMedian(float values[], size_t size) {
    std::sort(values, values + size);
    if (size % 2 == 0) {
        return (values[size / 2 - 1] + values[size / 2]) / 2.0f;
    }
    return values[size / 2];
}

int SensorManager::calculateMedianInt(int values[], size_t size) {
    std::sort(values, values + size);
    if (size % 2 == 0) {
        return (values[size / 2 - 1] + values[size / 2]) / 2;
    }
    return values[size / 2];
}
