#!/bin/bash
set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT/firmware-rust"

echo "=================================================="
echo " 🌿 Restoring Production Firmware (Release)..."
echo "=================================================="

cargo run --release
