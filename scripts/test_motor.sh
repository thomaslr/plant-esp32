#!/bin/bash
set -e

# Run the Multi-Level Pump Motor Diagnostic Test on the connected ESP32
# Automatically returns to the terminal prompt once the test completes!
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT/firmware-rust"

echo "=========================================================="
echo " 💧 Running Multi-Level PWM Pump Motor Diagnostic..."
echo " Target: Pump MOSFET Gate on GPIO 10 (5 kHz Hardware PWM)"
echo " Auto-exits back to prompt when all 5 levels complete."
echo " Press Ctrl+C at any time to abort."
echo "=========================================================="

# Auto-exit cleanly when the test completion banner is received
cargo run --example motor_test | awk '/Multi-Level PWM Test Completed/ { print; exit } 1'

