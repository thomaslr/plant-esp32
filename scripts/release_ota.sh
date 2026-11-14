#!/usr/bin/env bash
# Release + OTA helper for the Plant ESP32 firmware.
#
# Usage:
#   scripts/release_ota.sh [-v X.Y.Z] [-H plants2.local] [-m "changelog line"] [--ota-only]
#
#   (default)   bump patch version, test, commit, tag, push, wait for GitHub build,
#               download plant-firmware.bin and upload it to the device (POST /api/ota/upload).
#   -v          explicit version (default: latest tag + 1 patch)
#   -H          device host/IP (default: plants2.local)
#   -m          changelog bullet (default: generic)
#   --ota-only  skip versioning; OTA the latest GitHub release to the device
set -euo pipefail
cd "$(dirname "$0")/.."

HOST="plants2.local"; VERSION=""; NOTE="Firmware update."; OTA_ONLY=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    -v) VERSION="${2#v}"; shift 2;;
    -H) HOST="$2"; shift 2;;
    -m) NOTE="$2"; shift 2;;
    --ota-only) OTA_ONLY=1; shift;;
    *) echo "unknown arg $1"; exit 1;;
  esac
done

if [[ $OTA_ONLY -eq 0 ]]; then
  if [[ -z "$VERSION" ]]; then
    last=$(git describe --tags --abbrev=0); last=${last#v}
    IFS=. read -r a b c <<<"$last"; VERSION="$a.$b.$((c+1))"
  fi
  echo "==> Releasing v$VERSION"
  sed -i '' -E "s/^version = \".*\"/version = \"$VERSION\"/" \
    firmware-rust/plant-firmware/Cargo.toml firmware-rust/plant-core/Cargo.toml
  sed -i '' -E "s/(id=\"version-badge\"[^>]*>)v[0-9]+\.[0-9]+\.[0-9]+/\1v$VERSION/" data/index.html
  python3 - "$VERSION" "$NOTE" <<'E'
import sys,datetime
v,n=sys.argv[1:3];p='CHANGELOG.md';s=open(p).read()
marker="\n## ["
i=s.index(marker)
s=s[:i]+f"\n## [{v}] - {datetime.date.today()}\n\n- {n}\n\n---\n"+s[i:]
open(p,'w').write(s)
E
  (cd firmware-rust && cargo test --package plant-core --target aarch64-apple-darwin)
  git add -A firmware-rust data CHANGELOG.md
  git commit -m "chore: bump firmware to v$VERSION"
  git tag "v$VERSION"
  git push origin main
  git push origin "v$VERSION"
  sleep 15
  run_id=$(gh run list --workflow "Build & Release Firmware" --limit 1 --json databaseId -q '.[0].databaseId')
  gh run watch "$run_id" --exit-status
  TAG="v$VERSION"
else
  TAG=$(gh release view --json tagName -q .tagName)
fi

echo "==> Downloading plant-firmware.bin from $TAG"
tmp=$(mktemp -d)
gh release download "$TAG" -p plant-firmware.bin -D "$tmp"

echo "==> Uploading to http://$HOST/api/ota/upload"
curl --fail -sS -m 300 -X POST --data-binary @"$tmp/plant-firmware.bin" \
  -H "Content-Type: application/octet-stream" "http://$HOST/api/ota/upload"
echo
echo "==> Waiting for reboot..."
sleep 20
curl -sS -m 10 "http://$HOST/api/ota/status" || echo "device not back yet"
echo
