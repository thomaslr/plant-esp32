#!/bin/bash
set -e

# Run the 1 Hz Soil Sensor Diagnostic Tool on the connected ESP32
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT/firmware-rust"

echo "=================================================="
echo " 🌱 Flashing Soil Moisture Diagnostics to ESP32..."
echo " Press Ctrl+C in this console at any time to stop."
echo "=================================================="

cargo run --example soil_test
