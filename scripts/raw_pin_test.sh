#!/bin/bash
set -e

# Flash and run the pure raw hardware pin exerciser
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT/firmware-rust"

echo "========================================================"
echo " 🛠️ Flashing Raw Pin Test (No WiFi, No Sensors, No Super)"
echo " Press Ctrl+C at any time to stop."
echo "========================================================"

cargo run --example raw_pin_test
