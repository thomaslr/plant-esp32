---
name: version_fw
description: Bump firmware version, update Cargo.toml and UI assets, create a git tag, and trigger a release build on GitHub Actions for the Plant ESP32 firmware. Trigger when the user mentions "version_fw", "bump version", "tag version", "build release", or asks to release/version the firmware.
license: Apache-2.0
metadata:
  version: v1
  publisher: user
---

# Firmware Versioning & Release Workflow (`version_fw`)

This skill automates the complete lifecycle for releasing a new version of the Plant ESP32 firmware: updating package versions, synchronizing UI badges, tagging git commits, and triggering containerized builds on GitHub Actions.

## When to Run
Run this procedure whenever the user asks to:
- Use `/version_fw` or mentions `version_fw`
- Bump the firmware version (patch, minor, or major)
- Create a new release tag and build/publish the binary on GitHub

---

## Step-by-Step Procedure

### 1. Determine Target Version
1. Query the current version from `firmware-rust/plant-firmware/Cargo.toml` and latest git tag:
   ```bash
   git describe --tags --abbrev=0
   ```
2. If the user provided an explicit version (e.g., `v0.2.5` or `0.3.0`), sanitize it to ensure `X.Y.Z` format for Cargo and `vX.Y.Z` for the git tag.
3. If no version was specified, automatically calculate the next semantic patch bump (e.g. `0.2.4` → `0.2.5`).

### 2. Update Files
Update the version string and document release highlights across all required files:
- **`firmware-rust/plant-firmware/Cargo.toml`**: `version = "X.Y.Z"`
- **`firmware-rust/plant-core/Cargo.toml`**: `version = "X.Y.Z"`
- **`data/index.html`**: `<span class="badge" id="version-badge" ...>vX.Y.Z 📈</span>`
- **`CHANGELOG.md`**: Add `## [X.Y.Z] - YYYY-MM-DD` section with curated bulleted highlights (Safety & Bug Fixes, Features, Enhancements). The GitHub release workflow automatically extracts this section to populate the release notes.

### 3. Run Pre-Flight Tests
Ensure no regressions exist in core sensor math or safety supervisors:
```bash
cd firmware-rust && cargo test --package plant-core --target aarch64-apple-darwin
```

### 4. Commit and Tag
Stage all working changes and create a git tag:
```bash
git add firmware-rust/plant-firmware/Cargo.toml firmware-rust/plant-core/Cargo.toml data/index.html
# Include any pending code changes (such as led.rs, web.rs, etc.)
git commit -m "chore: bump firmware to vX.Y.Z"
git tag vX.Y.Z
```

### 5. Push to GitHub
Push both the branch and the new tag to GitHub:
```bash
git push origin main
git push origin vX.Y.Z
```
*Note: Pushing `v*` tags automatically triggers the `.github/workflows/build-and-release.yml` workflow, which executes inside the pre-built GHCR Docker container (`ghcr.io/thomaslr/plant-esp32-build:2026-11-05`).*

### 6. Monitor GitHub Action Build
1. Identify the triggered workflow run:
   ```bash
   gh run list --workflow "Build & Release Firmware" --limit 2
   ```
2. Monitor run status until completion:
   ```bash
   gh run watch <RUN_ID>
   ```
3. Verify that the GitHub Release `vX.Y.Z` contains both assets:
   - `plant-firmware.bin`
   - `version.json`

### 7. Notify User
Report the outcome to the user with clickable links:
- GitHub Release URL: `https://github.com/thomaslr/plant-esp32/releases/tag/vX.Y.Z`
- Remind the user that the firmware is ready to flash via the **"Check GitHub Update"** button on the web dashboard.
