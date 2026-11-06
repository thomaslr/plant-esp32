#!/bin/bash
set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT/firmware-rust"

echo "=================================================="
echo " 🌿 Restoring Production Firmware (Release)..."
echo "=================================================="

cargo build --release
espflash flash --partition-table plant-firmware/partitions.csv target/riscv32imc-esp-espidf/release/plant-firmware --monitor
