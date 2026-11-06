#!/bin/bash
set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT/firmware-rust"

echo "========================================================"
echo " 🔍 Running Pin Identification Test..."
echo " Press Ctrl+C at any time to stop."
echo "========================================================"

cargo run --example pin_identify
