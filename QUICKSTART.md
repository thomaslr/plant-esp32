# 🚀 Quick Start: First-Time Setup & Flashing (ESP32-C3 SuperMini)

This guide walks you through setting up and flashing an **ESP32-C3 SuperMini** for the **Plant Monitor & Automated Irrigation Controller** project.

> [!TIP]
> **No Development Environment Needed!**
> New users do **not** need to install Rust, Git, Python, or the ESP-IDF toolchain. You can flash the board directly from your web browser using a pre-built factory binary.

---

## 📦 1. Pre-Built Firmware Binaries

Every GitHub release provides two distinct binaries on the **[GitHub Releases Page](https://github.com/thomaslr/plant-esp32/releases)**:

| Binary File | Flash Address | Purpose & Usage |
| :--- | :---: | :--- |
| **`plant-firmware-factory.bin`** *(~4.0 MB / ~1.8 MB)* | **`0x0`** | **Initial Setup / First-Time Flashing:** Merged factory binary containing the ESP32-C3 2nd-stage bootloader, Dual-OTA partition table (`ota_0` and `ota_1`, 1984 KB each), and application code. Flashes blank chips in one shot. |
| **`plant-firmware.bin`** *(~1.7 MB)* | OTA Partition | **Wi-Fi Updates (OTA):** Standalone application payload uploaded via the web interface (`http://plants1.local`) or downloaded automatically during remote updates. |

---

## ⚠️ 2. Pre-Flashing Hardware Checklist

Before plugging in your board:

1. **Certified USB-C Data Cable:**
   * Many USB-C cables bundled with consumer electronics are **charge-only** (missing $D+$ and $D-$ data lines).
   * The ESP32-C3 SuperMini uses its internal native USB-Serial/JTAG controller, which requires a full USB data cable.
2. **Strapping Pin Precautions:**
   * **GPIO 2:** Strapping pin. Our PCB leaves it unconnected / High-Z to guarantee zero boot hangs.
   * **GPIO 8:** Controls the onboard blue status LED (Active LOW).
   * **GPIO 9:** Onboard `BOOT` tactile switch.
3. **Putting ESP32-C3 SuperMini into Download Boot Mode:**
   If your operating system or web browser does not auto-detect the serial port:
   * Press and hold the **`BOOT`** button (GPIO 9).
   * Press and release the **`RST`** button (or plug in the USB cable while holding `BOOT`).
   * Release the **`BOOT`** button. The chip is now in ROM download mode.

---

## 💻 3. Flashing Options

### Option 1: 1-Click In-Browser Web Flasher (Recommended)

No software installation required. Compatible with **Google Chrome** and **Microsoft Edge** (via Web Serial API):

1. Connect your ESP32-C3 SuperMini via USB.
2. Put the board into download mode (Hold `BOOT` $\rightarrow$ tap `RST` $\rightarrow$ release `BOOT`).
3. Open either web flasher in Chrome or Edge:
   * **[Espressif Web Flasher (esptool-js)](https://espressif.github.io/esptool-js/)** *(Official)*
   * **[Adafruit WebSerial ESPTool](https://adafruit.github.io/Adafruit_WebSerial_ESPTool/)**
4. Set the Baud Rate to **`460800`** and click **Connect**.
5. Select your ESP32-C3 port from the browser prompt (typically appears as *USB JTAG/serial debug unit* or *usbmodem*).
6. In the file upload section:
   * **File:** Select `plant-firmware-factory.bin` downloaded from [Releases](https://github.com/thomaslr/plant-esp32/releases).
   * **Flash Address:** **`0x0`** (or `0x0000`).
7. Click **Program / Flash**.
8. Once the progress bar reaches 100%, press the **`RST`** button once to boot the controller!

---

### Option 2: Single-Command USB CLI (`esptool.py` or `espflash`)

If you have Python or the Rust `espflash` utility installed, you can flash the merged factory image directly to address `0x0`:

```bash
# Using esptool.py (Python)
esptool.py --chip esp32c3 write_flash 0x0 plant-firmware-factory.bin

# Or using espflash (Rust CLI)
espflash write-bin 0x0 plant-firmware-factory.bin
```

Press **`RST`** once when complete.

---

### Option 3: Compiling & Flashing from Source (Rust Nightly)

For developers actively modifying the codebase:

```bash
# Navigate to the firmware package
cd firmware-rust/plant-firmware

# Compile and flash via .cargo/config.toml runner
cargo run --release
```

*(The workspace `.cargo/config.toml` automatically invokes `espflash flash --partition-table plant-firmware/partitions.csv --monitor`, provisioning the Dual-OTA partition table, building the bootloader, flashing the app, and launching the serial monitor).*

---

## 📶 4. First-Time Wi-Fi Provisioning (SoftAP & Captive Portal)

On first boot, the ESP32-C3 has no saved Wi-Fi credentials in NVS, so it automatically launches its configuration hotspot:
$$\mathbf{PlantMonitor\text{-}Setup\text{-}XXXX}$$
*(where `XXXX` is the last 4 hex characters of your ESP32's MAC address).*

### 📲 Instant Smartphone Setup with QR Codes

> [!NOTE]
> **Cellular Data Interference on iOS & Android:**
> Because the initial ESP32 setup hotspot does not have internet access, modern smartphones often prioritize 5G/4G cellular data and will either suppress the captive portal pop-up or route `192.168.4.1` out over the cellular network.
>
> **Quick Fix:** Scan the QR code below or temporarily toggle **Cellular Data OFF** in your phone settings while configuring Wi-Fi.

| 1. Scan to Join Wi-Fi | 2. Scan to Open Setup Portal |
| :---: | :---: |
| <img src="docs/assets/qr-setup-wifi.svg" width="180" height="180" alt="Connect to Setup Wi-Fi"> | <img src="docs/assets/qr-setup-url.svg" width="180" height="180" alt="Open Setup Webpage"> |
| **Network:** `PlantMonitor-Setup` *(Open / No Password)* | **URL:** [`http://192.168.4.1`](http://192.168.4.1) |

1. **Connect to Hotspot:** Point your smartphone camera at the first QR code to join (or select `PlantMonitor-Setup-XXXX` in Wi-Fi settings).
2. **Open Setup Portal:** Scan the second QR code or navigate your browser to **[http://192.168.4.1](http://192.168.4.1)**.
3. **Configure Wi-Fi:**
   * Select your 2.4 GHz home Wi-Fi SSID from the dropdown list.
   * Enter your Wi-Fi password (use the **Show** toggle to verify).
   * Click **Save & Connect**.
4. **Access Web Dashboard:**
   * The device will reboot, join your home router, and announce itself over mDNS.
   * Open: **[http://plants1.local](http://plants1.local)** *(or check your router DHCP client list)*.
5. **Install as PWA (Optional):**
   * On iOS Safari: Tap **Share** $\rightarrow$ **Add to Home Screen**.
   * On Android Chrome or Desktop: Tap the **Install** icon in the URL bar.

---

## 🔄 5. Future Over-The-Air (OTA) Updates

Once your board is running, you never need to connect a USB cable again:
1. Open the web dashboard at `http://plants1.local`.
2. Click **"Check for Updates"** under Diagnostics / System.
3. The dashboard queries latest GitHub release notes, prompts for confirmation, and streams `plant-firmware.bin` directly into the standby OTA slot.
4. On reboot, the bootloader automatically validates the new firmware. If anything goes wrong, it rolls back safely to the previous working slot.

---

## 🔘 6. Hardware Recovery & Factory Reset (`BOOT` Button)

If you ever change your Wi-Fi router, enter the wrong password, or give the plant monitor to a friend:
* **Press and hold the onboard `BOOT` button (GPIO 9) for 5 seconds.**
* The onboard blue LED will strobe rapidly.
* All saved Wi-Fi credentials will be cleared from NVS flash and the board will automatically reboot into the `PlantMonitor-Setup-XXXX` provisioning portal.
* **No computer, terminal, or USB cable required!**
