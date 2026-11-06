#!/bin/bash
set -e

# Run the Live Soil & Ultrasonic Sensor Diagnostic Tool on the connected ESP32
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT/firmware-rust"

echo "=========================================================="
echo " 🌿 Flashing Live Sensor Verification Diagnostic to ESP32..."
echo " Target: Soil Moisture (GPIO 3/21) & Ultrasonic (GPIO 7/20)"
echo " Press Ctrl+C in this console at any time to stop."
echo "=========================================================="

cargo run --example sensor_test
