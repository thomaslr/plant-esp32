#!/bin/bash
set -e

# Run the 2s ON / 2s OFF TB2 Pin 2 (GPIO 21 / GPIO 5) Voltmeter Diagnostic Tool on the connected ESP32
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT/firmware-rust"

echo "========================================================"
echo " ⚡ Flashing TB2 Pin 2 Voltmeter Toggle Diagnostic (2s / 2s)..."
echo " Target: TB2 Pin 2 (Switched VCC on GPIO 21)"
echo " Press Ctrl+C in this console at any time to stop."
echo "========================================================"

cargo run --example gpio5_toggle
